use std::collections::HashMap;
use std::sync::Arc;

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
const SUPPORTED_PROTOCOL_VERSION: &str = "1.0";

pub struct McpService {
    parliament_client: Arc<ParliamentClient>,
    research_service: Arc<ResearchService>,
    utilities_service: Arc<DateTimeService>,
    tool_schemas: Vec<ToolDefinition>,
    argument_validators: HashMap<String, JSONSchema>,
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
                    tracing::error!(tool = %name, error = %err, "failed to compile JSON schema for tool arguments");
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
        }
    }

    pub async fn handle_jsonrpc(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        if request.jsonrpc != JSON_RPC_VERSION {
            return Err(self.invalid_request_response(
                request.id,
                -32600,
                format!("unsupported jsonrpc version: {}", request.jsonrpc),
            ));
        }

        if request.id.is_null() {
            return Err(self.invalid_request_response(
                request.id,
                -32600,
                "request id must not be null".to_string(),
            ));
        }

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "list_tools" | "tools/list" => self.handle_list_tools(request).await,
            "call_tool" | "tools/call" => self.handle_call_tool(request).await,
            other => Err(self.invalid_request_response(
                request.id,
                -32601,
                format!("unknown method: {other}"),
            )),
        }
    }

    async fn handle_initialize(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let params = match request.params {
            Some(value) => serde_json::from_value::<InitializeParams>(value).map_err(|err| {
                self.invalid_request_response(
                    request.id.clone(),
                    -32602,
                    format!("invalid initialize params: {err}"),
                )
            })?,
            None => {
                return Err(self.invalid_request_response(
                    request.id,
                    -32602,
                    "missing initialize params".to_string(),
                ));
            }
        }; 

        if params.protocol_version != SUPPORTED_PROTOCOL_VERSION {
            return Err(self.invalid_request_response(
                request.id,
                -32600,
                format!(
                    "unsupported protocolVersion: {}",
                    params.protocol_version
                ),
            ));
        }

        if !params.capabilities.is_object() {
            return Err(self.invalid_request_response(
                request.id,
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

        let result = json!({
            "serverInfo": {
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
                "description": "Model Context Protocol server for UK Parliament research"
            },
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            }
        });

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: request.id,
            result,
        })
    }

    async fn handle_list_tools(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let params = match request.params {
            Some(value) => serde_json::from_value::<ListToolsParams>(value).map_err(|err| {
                self.invalid_request_response(
                    request.id.clone(),
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
                request.id.clone(),
                format!("failed to serialize tools: {err}"),
            )
        })?;

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: request.id,
            result,
        })
    }

    async fn handle_call_tool(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcSuccess, JsonRpcErrorResponse> {
        let params_value = request.params.ok_or_else(|| {
            self.invalid_request_response(
                request.id.clone(),
                -32602,
                "missing call_tool params".to_string(),
            )
        })?;

        let params = serde_json::from_value::<CallToolParams>(params_value).map_err(|err| {
            self.invalid_request_response(
                request.id.clone(),
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
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_core_dataset(&self.parliament_client, args).await
            }
            "parliament.fetch_bills" => {
                let args = self.deserialize_arguments::<FetchBillsArgs>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_bills(&self.parliament_client, args).await
            }
            "parliament.fetch_legislation" => {
                let args = self.deserialize_arguments::<FetchLegislationArgs>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_legislation(&self.parliament_client, args).await
            }
            "parliament.fetch_mp_activity" => {
                let args = self.deserialize_arguments::<FetchMpActivityArgs>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_mp_activity(&self.parliament_client, args).await
            }
            "parliament.fetch_mp_voting_record" => {
                let args = self.deserialize_arguments::<FetchMpVotingRecordArgs>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_fetch_mp_voting_record(&self.parliament_client, args).await
            }
            "parliament.lookup_constituency_offline" => {
                let args = self.deserialize_arguments::<LookupConstituencyArgs>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_lookup_constituency_offline(&self.parliament_client, args).await
            }
            "parliament.search_uk_law" => {
                let args = self.deserialize_arguments::<SearchUkLawArgs>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                handle_search_uk_law(&self.parliament_client, args).await
            }
            "research.run" => {
                let args = self.deserialize_arguments::<ResearchRequestDto>(
                    &request.id,
                    tool_name.as_str(),
                    arguments.clone(),
                )?;
                match handle_run_research(&self.research_service, args).await {
                    Ok(result) => serde_json::to_value(result)
                        .map_err(|err| AppError::internal(format!(
                            "failed to serialize research response: {err}"
                        ))),
                    Err(err) => Err(err),
                }
            }
            "utilities.current_datetime" => {
                let result = handle_current_datetime(&self.utilities_service);
                serde_json::to_value(result)
                    .map_err(|err| AppError::internal(format!(
                        "failed to serialize datetime payload: {err}"
                    )))
            }
            other => {
                return Err(self.invalid_request_response(
                    request.id,
                    -32601,
                    format!("unknown tool: {other}"),
                ));
            }
        };

        match call_result {
            Ok(payload) => self.build_tool_success(request.id, payload),
            Err(AppError::BadRequest { message }) => Err(self.invalid_request_response(
                request.id,
                -32602,
                message,
            )),
            Err(error) => Ok(self.tool_execution_error(request.id, tool_name.as_str(), error)),
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
                id.clone(),
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
                    id.clone(),
                    -32602,
                    format!("invalid tool arguments: {message}"),
                ));
            }
        } else {
            tracing::debug!(tool = tool_name, "no validator registered for tool arguments");
        }

        serde_json::from_value::<T>(value).map_err(|err| {
            self.invalid_request_response(
                id.clone(),
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
                id.clone(),
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
                id.clone(),
                format!("failed to encode tool response: {err}"),
            )
        })?;

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            result,
        })
    }

    fn tool_execution_error(
        &self,
        id: Value,
        tool_name: &str,
        error: AppError,
    ) -> JsonRpcSuccess {
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
        id: Value,
        code: i32,
        message: String,
    ) -> JsonRpcErrorResponse {
        JsonRpcErrorResponse {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            error: JsonRpcError {
                code,
                message,
                data: None,
            },
        }
    }

    fn internal_error_response(&self, id: Value, message: String) -> JsonRpcErrorResponse {
        JsonRpcErrorResponse {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            error: JsonRpcError {
                code: -32000,
                message,
                data: None,
            },
        }
    }
}
