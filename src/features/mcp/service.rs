use std::sync::Arc;

use serde_json::{Value, json};

use crate::core::error::AppError;
use crate::features::mcp::dto::{
    CallToolParams, InitializeParams, JsonRpcError, JsonRpcErrorResponse, JsonRpcRequest,
    JsonRpcSuccess, ToolCallResult, ToolContent, ToolListResult,
};
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

pub struct McpService {
    parliament_client: Arc<ParliamentClient>,
    research_service: Arc<ResearchService>,
    utilities_service: Arc<DateTimeService>,
    tool_schemas: Vec<Value>,
}

impl McpService {
    pub fn new(
        parliament_client: Arc<ParliamentClient>,
        research_service: Arc<ResearchService>,
    ) -> Self {
        let tool_schemas = build_tool_schemas();
        let utilities_service = Arc::new(DateTimeService::new());

        Self {
            parliament_client,
            research_service,
            utilities_service,
            tool_schemas,
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

        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "list_tools" => self.handle_list_tools(request).await,
            "call_tool" => self.handle_call_tool(request).await,
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
        if let Some(params) = request.params {
            if params != Value::Object(Default::default()) {
                serde_json::from_value::<serde_json::Map<String, Value>>(params).map_err(
                    |err| {
                        self.invalid_request_response(
                            request.id.clone(),
                            -32602,
                            format!("invalid list_tools params: {err}"),
                        )
                    },
                )?;
            }
        }

        let result = serde_json::to_value(ToolListResult {
            tools: self.tool_schemas.clone(),
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

        let result_json = match params.name.as_str() {
            "parliament.fetch_core_dataset" => {
                let args = self
                    .deserialize_arguments::<FetchCoreDatasetArgs>(&request.id, params.arguments)?;
                handle_fetch_core_dataset(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "parliament.fetch_bills" => {
                let args =
                    self.deserialize_arguments::<FetchBillsArgs>(&request.id, params.arguments)?;
                handle_fetch_bills(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "parliament.fetch_legislation" => {
                let args = self
                    .deserialize_arguments::<FetchLegislationArgs>(&request.id, params.arguments)?;
                handle_fetch_legislation(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "parliament.fetch_mp_activity" => {
                let args = self
                    .deserialize_arguments::<FetchMpActivityArgs>(&request.id, params.arguments)?;
                handle_fetch_mp_activity(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "parliament.fetch_mp_voting_record" => {
                let args = self.deserialize_arguments::<FetchMpVotingRecordArgs>(
                    &request.id,
                    params.arguments,
                )?;
                handle_fetch_mp_voting_record(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "parliament.lookup_constituency_offline" => {
                let args = self.deserialize_arguments::<LookupConstituencyArgs>(
                    &request.id,
                    params.arguments,
                )?;
                handle_lookup_constituency_offline(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "parliament.search_uk_law" => {
                let args =
                    self.deserialize_arguments::<SearchUkLawArgs>(&request.id, params.arguments)?;
                handle_search_uk_law(&self.parliament_client, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?
            }
            "research.run" => {
                let args = self
                    .deserialize_arguments::<ResearchRequestDto>(&request.id, params.arguments)?;
                let result = handle_run_research(&self.research_service, args)
                    .await
                    .map_err(|err| self.tool_failure_response(request.id.clone(), err))?;
                serde_json::to_value(result).map_err(|err| {
                    self.internal_error_response(
                        request.id.clone(),
                        format!("failed to serialize research response: {err}"),
                    )
                })?
            }
            "utilities.current_datetime" => {
                let result = handle_current_datetime(&self.utilities_service);
                serde_json::to_value(result).map_err(|err| {
                    self.internal_error_response(
                        request.id.clone(),
                        format!("failed to serialize datetime payload: {err}"),
                    )
                })?
            }
            other => {
                return Err(self.invalid_request_response(
                    request.id,
                    -32601,
                    format!("unknown tool: {other}"),
                ));
            }
        };

        let result = serde_json::to_value(ToolCallResult {
            content: vec![ToolContent {
                content_type: "json".to_string(),
                json: result_json,
            }],
        })
        .map_err(|err| {
            self.internal_error_response(
                request.id.clone(),
                format!("failed to serialize tool result: {err}"),
            )
        })?;

        Ok(JsonRpcSuccess {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: request.id,
            result,
        })
    }

    fn deserialize_arguments<T>(&self, id: &Value, value: Value) -> Result<T, JsonRpcErrorResponse>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value::<T>(value).map_err(|err| {
            self.invalid_request_response(
                id.clone(),
                -32602,
                format!("invalid tool arguments: {err}"),
            )
        })
    }

    fn tool_failure_response(&self, id: Value, error: AppError) -> JsonRpcErrorResponse {
        let (code, message, data) = match error {
            AppError::BadRequest { message } => (-32602, message, None),
            AppError::Upstream { message, data } => (-32002, message, data),
            AppError::Configuration { message } | AppError::Internal { message } => {
                (-32000, message, None)
            }
        };

        JsonRpcErrorResponse {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            error: JsonRpcError {
                code,
                message,
                data,
            },
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

fn build_tool_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "parliament.fetch_core_dataset",
            "description": "Fetch data from UK Parliament core datasets (legacy Linked Data API) and the Members API.",
            "inputSchema": {
                "type": "object",
                "required": ["dataset"],
                "properties": {
                    "dataset": {"type": "string"},
                    "searchTerm": {"type": "string"},
                    "page": {"type": "integer", "minimum": 0},
                    "perPage": {"type": "integer", "minimum": 1, "maximum": 100},
                    "enableCache": {"type": "boolean"},
                    "fuzzyMatch": {"type": "boolean"},
                    "applyRelevance": {"type": "boolean"},
                    "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "parliament.fetch_bills",
            "description": "Search for UK Parliament bills via the versioned bills-api.parliament.uk service.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "searchTerm": {"type": "string"},
                    "house": {"type": "string", "enum": ["commons", "lords"]},
                    "session": {"type": "string"},
                    "parliamentNumber": {"type": "integer", "minimum": 1},
                    "enableCache": {"type": "boolean"},
                    "applyRelevance": {"type": "boolean"},
                    "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "parliament.fetch_legislation",
            "description": "Retrieve legislation metadata from legislation.gov.uk Atom feeds.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "year": {"type": "integer", "minimum": 1800},
                    "type": {"type": "string", "enum": ["all", "ukpga", "ukci", "ukla", "nisi"]},
                    "enableCache": {"type": "boolean"},
                    "applyRelevance": {"type": "boolean"},
                    "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "parliament.fetch_mp_activity",
            "description": "List recent activity (debates, questions, statements) for a given MP.",
            "inputSchema": {
                "type": "object",
                "required": ["mpId"],
                "properties": {
                    "mpId": {"type": "integer", "minimum": 1},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 50},
                    "enableCache": {"type": "boolean"}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "parliament.fetch_mp_voting_record",
            "description": "Summarise an MP's voting record, optionally filtering by date range or bill.",
            "inputSchema": {
                "type": "object",
                "required": ["mpId"],
                "properties": {
                    "mpId": {"type": "integer", "minimum": 1},
                    "fromDate": {"type": "string", "format": "date"},
                    "toDate": {"type": "string", "format": "date"},
                    "billId": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100},
                    "enableCache": {"type": "boolean"}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "parliament.lookup_constituency_offline",
            "description": "Resolve a postcode to its Westminster constituency using the bundled dataset.",
            "inputSchema": {
                "type": "object",
                "required": ["postcode"],
                "properties": {
                    "postcode": {"type": "string", "minLength": 2},
                    "enableCache": {"type": "boolean"}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "parliament.search_uk_law",
            "description": "Search the complete UK legislation corpus for laws, acts, and statutory instruments.",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": {"type": "string", "minLength": 1},
                    "legislationType": {"type": "string", "enum": ["primary", "secondary", "all"]},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 50},
                    "enableCache": {"type": "boolean"}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "research.run",
            "description": "Aggregate bills, debates, legislation, votes and party balance for a parliamentary topic.",
            "inputSchema": {
                "type": "object",
                "required": ["topic"],
                "properties": {
                    "topic": {"type": "string", "minLength": 1},
                    "billKeywords": {"type": "array", "items": {"type": "string"}},
                    "debateKeywords": {"type": "array", "items": {"type": "string"}},
                    "mpId": {"type": "integer", "minimum": 1},
                    "includeStateOfParties": {"type": "boolean"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 10}
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "utilities.current_datetime",
            "description": "Return the current UTC time alongside Europe/London local time.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
    ]
}
