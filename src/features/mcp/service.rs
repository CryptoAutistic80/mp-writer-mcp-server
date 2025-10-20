use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use jsonschema::JSONSchema;
use serde_json::{Value, json};

use crate::core::error::AppError;
use crate::features::mcp::dto::{
    CallToolParams, InitializeParams, JsonRpcError, JsonRpcErrorResponse, JsonRpcRequest,
    JsonRpcSuccess, ListToolsParams, ToolCallResult, ToolContent, ToolDefinition, ToolListResult,
};
use crate::features::mcp::schemas::build_tool_schemas;
use crate::features::parliament::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchLegislationArgs, FetchMpActivityArgs,
    FetchMpVotingRecordArgs, LookupConstituencyArgs, ParliamentClient, SearchUkLawArgs,
    handle_fetch_bills, handle_fetch_core_dataset, handle_fetch_legislation,
    handle_fetch_mp_activity, handle_fetch_mp_voting_record, handle_lookup_constituency_offline,
    handle_search_uk_law,
};
use crate::features::research::{ResearchRequestDto, ResearchService, handle_run_research};
use crate::features::utilities::{DateTimeService, handle_current_datetime};

const JSON_RPC_VERSION: &str = "2.0";
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-26", "2025-06-18", "2025-03-26", "1.1", "1.0"];
const PROTOCOL_VERSION_1_1_ALIASES: &[&str] = &["2025-06-26", "2025-06-18", "2025-03-26", "1.1"];

pub struct McpService {
    parliament_client: Arc<ParliamentClient>,
    research_service: Arc<ResearchService>,
    utilities_service: Arc<DateTimeService>,
    tool_schemas: Vec<ToolDefinition>,
    argument_validators: HashMap<String, JSONSchema>,
    negotiated_protocol: Mutex<Option<String>>,
    initialize_called: AtomicBool,
    client_ready: AtomicBool,
}

impl McpService {
    pub fn new(
        parliament_client: Arc<ParliamentClient>,
        research_service: Arc<ResearchService>,
    ) -> Self {
        let (tool_schemas, input_schemas) = build_tool_schemas();
        let mut argument_validators = HashMap::new();

        for (name, schema) in input_schemas {
            match JSONSchema::compile(&schema) {
                Ok(compiled) => {
                    argument_validators.insert(name, compiled);
                }
                Err(err) => {
                    tracing::error!(
                        tool = %name,
                        error = %err,
                        "failed to compile JSON schema for tool arguments"
                    );
                }
            }
        }
        let utilities_service = Arc::new(DateTimeService::new());

        Self {
            parliament_client,
            research_service,
            utilities_service,
            tool_schemas,
            argument_validators,
            negotiated_protocol: Mutex::new(None),
            initialize_called: AtomicBool::new(false),
            client_ready: AtomicBool::new(false),
        }
    }

    pub fn negotiated_protocol_version(&self) -> Option<String> {
        match self.negotiated_protocol.lock() {
            Ok(guard) => guard.clone(),
            Err(error) => {
                tracing::error!(error = %error, "protocol version mutex poisoned");
                None
            }
        }
    }

    pub async fn handle_jsonrpc(
        &self,
        request: JsonRpcRequest,
        header_protocol_version: Option<String>,
    ) -> Result<Option<JsonRpcSuccess>, JsonRpcErrorResponse> {
        let JsonRpcRequest {
            jsonrpc,
            id,
            method,
            params,
        } = request;

        if jsonrpc != JSON_RPC_VERSION {
            return Err(self.invalid_request_response(
                id,
                -32600,
                format!("unsupported jsonrpc version: {jsonrpc}"),
            ));
        }

        match method.as_str() {
            "initialize" => {
                let request_id = self.require_request_id(&id, "initialize")?;
                let header_version = header_protocol_version.clone().ok_or_else(|| {
                    self.invalid_request_response(
                        Some(request_id.clone()),
                        -32600,
                        "initialize requires MCP-Protocol-Version header".to_string(),
                    )
                })?;
                self.handle_initialize(request_id, params, header_version)
                    .await
                    .map(Some)
            }
            "notifications/initialized" | "initialized" => {
                // Relaxed behavior: allow missing MCP-Protocol-Version on the initialized notification
                // to maintain compatibility with clients that omit headers on notifications. If the
                // header is present, still validate it against the negotiated version.
                if header_protocol_version.is_some() {
                    self.ensure_protocol_header(header_protocol_version.as_deref(), &id)?;
                }
                self.handle_initialized_notification(method.as_str());
                Ok(None)
            }
            "list_tools" | "tools/list" => {
                let request_id = self.require_request_id(&id, "tools/list")?;
                let id_for_header = Some(request_id.clone());
                self.ensure_protocol_header(header_protocol_version.as_deref(), &id_for_header)?;
                self.ensure_ready(Some(request_id.clone()))?;
                self.handle_list_tools(request_id, params).await.map(Some)
            }
            "call_tool" | "tools/call" => {
                let request_id = self.require_request_id(&id, "tools/call")?;
                let id_for_header = Some(request_id.clone());
                self.ensure_protocol_header(header_protocol_version.as_deref(), &id_for_header)?;
                self.ensure_ready(Some(request_id.clone()))?;
                self.handle_call_tool(request_id, params).await.map(Some)
            }
            "ping" => {
                let request_id = self.require_request_id(&id, "ping")?;
                let id_for_header = Some(request_id.clone());
                self.ensure_protocol_header(header_protocol_version.as_deref(), &id_for_header)?;
                self.ensure_initialized(Some(request_id.clone()))?;
                self.handle_ping(request_id).map(Some)
            }
            other => {
                Err(self.invalid_request_response(id, -32601, format!("unknown method: {other}")))
            }
        }
    }

    async fn handle_initialize(
        &self,
        id: Value,
        params: Option<Value>,
        header_protocol_version: String,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let params = match params {
            Some(value) => serde_json::from_value::<InitializeParams>(value).map_err(|err| {
                self.invalid_request_response(
                    Some(id.clone()),
                    -32602,
                    format!("invalid initialize params: {err}"),
                )
            })?,
            None => {
                return Err(self.invalid_request_response(
                    Some(id.clone()),
                    -32602,
                    "missing initialize params".to_string(),
                ));
            }
        };

        if !Self::protocol_headers_compatible(
            params.protocol_version.as_str(),
            header_protocol_version.as_str(),
        ) {
            return Err(self.invalid_request_response(
                Some(id.clone()),
                -32600,
                format!(
                    "MCP-Protocol-Version header mismatch: payload requested {} but header provided {}",
                    params.protocol_version, header_protocol_version
                ),
            ));
        }

        let negotiated = self
            .negotiate_protocol_version(&params.protocol_version)
            .ok_or_else(|| {
                self.invalid_request_response(
                    Some(id.clone()),
                    -32600,
                    format!("unsupported protocolVersion: {}", params.protocol_version),
                )
            })?;

        if !params.capabilities.is_object() {
            return Err(self.invalid_request_response(
                Some(id.clone()),
                -32602,
                "capabilities must be an object".to_string(),
            ));
        }

        tracing::info!(
            client = %params.client_info.name,
            version = %params.client_info.version,
            "client initialized"
        );
        tracing::debug!(
            protocol = %params.protocol_version,
            capabilities = ?params.capabilities,
            "initialize payload"
        );

        match self.negotiated_protocol.lock() {
            Ok(mut guard) => {
                *guard = Some(negotiated.clone());
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to record negotiated protocol version");
            }
        }

        self.initialize_called.store(true, Ordering::SeqCst);
        self.client_ready.store(false, Ordering::SeqCst);

        let result = json!({
            "protocolVersion": negotiated,
            "serverInfo": {
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "description": "Model Context Protocol server for UK Parliament research"
            },
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "instructions": "Call the initialized notification after a successful initialize response, then use tools/list to discover available tools."
        });

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result,
        })
    }

    async fn handle_list_tools(
        &self,
        id: Value,
        params: Option<Value>,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let params = match params {
            Some(value) => serde_json::from_value::<ListToolsParams>(value).map_err(|err| {
                self.invalid_request_response(
                    Some(id.clone()),
                    -32602,
                    format!("invalid tools/list params: {err}"),
                )
            })?,
            None => ListToolsParams::default(),
        };

        if let Some(cursor) = params.cursor {
            tracing::info!(
                %cursor,
                "received pagination cursor for tools/list; pagination is not supported"
            );
        }

        let tools = self.tool_schemas.clone();

        let result = serde_json::to_value(ToolListResult {
            tools,
            next_cursor: None,
        })
        .map_err(|err| {
            self.internal_error_response(
                Some(id.clone()),
                format!("failed to serialize tools: {err}"),
            )
        })?;

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result,
        })
    }

    async fn handle_call_tool(
        &self,
        id: Value,
        params: Option<Value>,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let params_value = params.ok_or_else(|| {
            self.invalid_request_response(
                Some(id.clone()),
                -32602,
                "missing call_tool params".to_string(),
            )
        })?;

        let params = serde_json::from_value::<CallToolParams>(params_value).map_err(|err| {
            self.invalid_request_response(
                Some(id.clone()),
                -32602,
                format!("invalid call_tool params: {err}"),
            )
        })?;

        let tool_name = params.name;
        let arguments = if params.arguments.is_null() {
            json!({})
        } else {
            params.arguments
        };

        let call_result: Result<Value, AppError> = match tool_name.as_str() {
            "parliament.fetch_core_dataset" => {
                let args = self.deserialize_arguments::<FetchCoreDatasetArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_core_dataset(&self.parliament_client, args).await
            }
            "parliament.fetch_bills" => {
                let args = self.deserialize_arguments::<FetchBillsArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_bills(&self.parliament_client, args).await
            }
            "parliament.fetch_legislation" => {
                let args = self.deserialize_arguments::<FetchLegislationArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_legislation(&self.parliament_client, args).await
            }
            "parliament.fetch_mp_activity" => {
                let args = self.deserialize_arguments::<FetchMpActivityArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_mp_activity(&self.parliament_client, args).await
            }
            "parliament.fetch_mp_voting_record" => {
                let args = self.deserialize_arguments::<FetchMpVotingRecordArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_mp_voting_record(&self.parliament_client, args).await
            }
            "parliament.lookup_constituency_offline" => {
                let args = self.deserialize_arguments::<LookupConstituencyArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_lookup_constituency_offline(&self.parliament_client, args).await
            }
            "parliament.search_uk_law" => {
                let args = self.deserialize_arguments::<SearchUkLawArgs>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_search_uk_law(&self.parliament_client, args).await
            }
            "research.run" => {
                let args = self.deserialize_arguments::<ResearchRequestDto>(
                    &id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                match handle_run_research(&self.research_service, args).await {
                    Ok(result) => serde_json::to_value(result).map_err(|err| {
                        AppError::internal(format!("failed to serialize research response: {err}"))
                    }),
                    Err(err) => Err(err),
                }
            }
            "utilities.current_datetime" => {
                let result = handle_current_datetime(&self.utilities_service);
                serde_json::to_value(result).map_err(|err| {
                    AppError::internal(format!("failed to serialize datetime payload: {err}"))
                })
            }
            other => {
                return Err(self.invalid_request_response(
                    Some(id),
                    -32601,
                    format!("unknown tool: {other}"),
                ));
            }
        };

        match call_result {
            Ok(payload) => self.build_tool_success(id, payload),
            Err(AppError::BadRequest { message }) => {
                Err(self.invalid_request_response(Some(id), -32602, message))
            }
            Err(error) => Ok(self.tool_execution_error(id, tool_name.as_str(), error)),
        }
    }

    fn deserialize_arguments<T>(
        &self,
        id: &Value,
        tool_name: &str,
        value: Value,
    ) -> Result<T, JsonRpcErrorResponse>
    where
        T: serde::de::DeserializeOwned,
    {
        if !value.is_object() {
            return Err(self.invalid_request_response(
                Some(id.clone()),
                -32602,
                "tool arguments must be an object".to_string(),
            ));
        }

        if let Some(validator) = self.argument_validators.get(tool_name) {
            if let Err(errors) = validator.validate(&value) {
                let message = errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");

                return Err(self.invalid_request_response(
                    Some(id.clone()),
                    -32602,
                    format!("invalid tool arguments: {message}"),
                ));
            }
        } else {
            tracing::debug!(
                tool = tool_name,
                "no validator registered for tool arguments"
            );
        }

        serde_json::from_value::<T>(value).map_err(|err| {
            self.invalid_request_response(
                Some(id.clone()),
                -32602,
                format!("invalid tool arguments: {err}"),
            )
        })
    }

    fn build_tool_success(
        &self,
        id: Value,
        payload: Value,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let rendered = serde_json::to_string_pretty(&payload).map_err(|err| {
            self.internal_error_response(
                Some(id.clone()),
                format!("failed to render tool payload: {err}"),
            )
        })?;

        let tool_result = ToolCallResult {
            content: vec![ToolContent {
                kind: "text".to_string(),
                text: rendered,
            }],
            structured_content: Some(payload),
            is_error: None,
        };

        let result = serde_json::to_value(tool_result).map_err(|err| {
            self.internal_error_response(
                Some(id.clone()),
                format!("failed to encode tool response: {err}"),
            )
        })?;

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result,
        })
    }

    fn tool_execution_error(&self, id: Value, tool_name: &str, error: AppError) -> JsonRpcSuccess {
        let sanitized_message = self.describe_tool_error(tool_name, &error);
        tracing::warn!(tool = tool_name, message = %sanitized_message, "tool execution failed");
        tracing::debug!(tool = tool_name, error = ?error, "detailed tool execution failure");

        let fallback_message = sanitized_message.clone();
        let tool_result = ToolCallResult {
            content: vec![ToolContent {
                kind: "text".to_string(),
                text: sanitized_message,
            }],
            structured_content: None,
            is_error: Some(true),
        };

        let result = match serde_json::to_value(tool_result) {
            Ok(value) => value,
            Err(err) => {
                tracing::error!(
                    tool = tool_name,
                    error = %err,
                    "failed to encode tool error payload"
                );
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": fallback_message
                        }
                    ],
                    "isError": true
                })
            }
        };

        JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result,
        }
    }

    fn handle_initialized_notification(&self, method: &str) {
        if !self.initialize_called.load(Ordering::SeqCst) {
            tracing::warn!(
                method,
                "received {method} before initialize; ignoring notification"
            );
            return;
        }

        self.client_ready.store(true, Ordering::SeqCst);
        tracing::info!(method, "client signalled readiness via {method}");
    }

    fn ensure_ready(&self, id: Option<Value>) -> Result<(), JsonRpcErrorResponse> {
        if !self.initialize_called.load(Ordering::SeqCst) {
            return Err(self.invalid_request_response(
                id.clone(),
                -32002,
                "client must call initialize before invoking this method".to_string(),
            ));
        }

        if !self.client_ready.load(Ordering::SeqCst) {
            return Err(self.invalid_request_response(
                id,
                -32002,
                "client must send the initialized notification before invoking this method"
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn ensure_initialized(&self, id: Option<Value>) -> Result<(), JsonRpcErrorResponse> {
        if !self.initialize_called.load(Ordering::SeqCst) {
            return Err(self.invalid_request_response(
                id,
                -32002,
                "client must call initialize before invoking this method".to_string(),
            ));
        }

        Ok(())
    }

    fn handle_ping(&self, id: Value) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result: json!({
                "ok": true
            }),
        })
    }

    fn ensure_protocol_header(
        &self,
        header_protocol_version: Option<&str>,
        id: &Option<Value>,
    ) -> Result<(), JsonRpcErrorResponse> {
        let expected_version = match self.negotiated_protocol_version() {
            Some(version) => version,
            None => {
                return Err(self.invalid_request_response(
                    id.clone(),
                    -32002,
                    "client must call initialize before invoking this method".to_string(),
                ));
            }
        };

        match header_protocol_version {
            Some(value) if Self::protocol_headers_compatible(value, &expected_version) => Ok(()),
            Some(value) => Err(self.invalid_request_response(
                id.clone(),
                -32600,
                format!(
                    "MCP-Protocol-Version header mismatch: expected {expected_version}, received {value}"
                ),
            )),
            None => {
                tracing::warn!(
                    expected = %expected_version,
                    "MCP-Protocol-Version header missing; assuming negotiated version"
                );
                Ok(())
            }
        }
    }

    fn require_request_id(
        &self,
        id: &Option<Value>,
        method: &str,
    ) -> Result<Value, JsonRpcErrorResponse> {
        match id {
            Some(value) if !value.is_null() => Ok(value.clone()),
            _ => Err(self.invalid_request_response(
                id.clone(),
                -32600,
                format!("{method} requires a non-null id"),
            )),
        }
    }

    fn negotiate_protocol_version(&self, requested: &str) -> Option<String> {
        // If the exact version is supported, return it.
        if SUPPORTED_PROTOCOL_VERSIONS.iter().any(|v| *v == requested) {
            return Some(requested.to_string());
        }

        // Backward/forward-compatibility mapping:
        // Treat the date-based protocol tag as equivalent to 1.1 for capability purposes.
        if PROTOCOL_VERSION_1_1_ALIASES.iter().any(|alias| *alias == requested) {
            return Some("1.1".to_string());
        }

        None
    }

    fn protocol_headers_compatible(a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }

        // Consider these aliases equivalent for header checks to avoid needless client failures
        // when a client pins a date-based header but the server negotiates a semantic version.
        PROTOCOL_VERSION_1_1_ALIASES.contains(&a) && PROTOCOL_VERSION_1_1_ALIASES.contains(&b)
    }

    fn describe_tool_error(&self, tool_name: &str, error: &AppError) -> String {
        match error {
            AppError::Upstream { data, .. } => {
                if let Some(status) = data
                    .as_ref()
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_u64())
                {
                    format!("Upstream service responded with HTTP {status}")
                } else {
                    format!("Upstream service request for {tool_name} failed")
                }
            }
            AppError::Configuration { .. } => {
                format!("Server configuration prevented running {tool_name}")
            }
            AppError::Internal { .. } => {
                format!("Internal error while executing {tool_name}")
            }
            AppError::BadRequest { message } => message.clone(),
        }
    }

    fn invalid_request_response(
        &self,
        id: Option<Value>,
        code: i32,
        message: String,
    ) -> JsonRpcErrorResponse {
        JsonRpcErrorResponse {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: id.unwrap_or(Value::Null),
            error: JsonRpcError {
                code,
                message,
                data: None,
            },
        }
    }

    fn internal_error_response(&self, id: Option<Value>, message: String) -> JsonRpcErrorResponse {
        JsonRpcErrorResponse {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: id.unwrap_or(Value::Null),
            error: JsonRpcError {
                code: -32000,
                message,
                data: None,
            },
        }
    }
}
