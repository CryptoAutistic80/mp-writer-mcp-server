use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcSuccess {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub error: JsonRpcError,
}

#[derive(Debug, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
    pub capabilities: Value,
}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListToolsParams {
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ToolListResult {
    pub tools: Vec<ToolDefinition>,
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ToolContent>,
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchToolTarget {
    UkLaw,
    Bills,
    Dataset,
}

#[derive(Debug, Deserialize)]
pub struct SearchToolArgs {
    pub target: SearchToolTarget,
    pub query: Option<String>,
    pub dataset: Option<String>,
    #[serde(rename = "legislationType")]
    pub legislation_type: Option<String>,
    pub limit: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
    #[serde(rename = "applyRelevance")]
    pub apply_relevance: Option<bool>,
    #[serde(rename = "relevanceThreshold")]
    pub relevance_threshold: Option<f32>,
    #[serde(rename = "fuzzyMatch")]
    pub fuzzy_match: Option<bool>,
    #[serde(rename = "house")]
    pub house: Option<String>,
    pub session: Option<String>,
    #[serde(rename = "parliamentNumber")]
    pub parliament_number: Option<u32>,
    #[serde(rename = "page")]
    pub page: Option<u32>,
    #[serde(rename = "perPage")]
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FetchToolTarget {
    CoreDataset,
    Bills,
    Legislation,
    MpActivity,
    MpVotingRecord,
    Constituency,
}

#[derive(Debug, Deserialize)]
pub struct FetchToolArgs {
    pub target: FetchToolTarget,
    pub dataset: Option<String>,
    #[serde(rename = "searchTerm")]
    pub search_term: Option<String>,
    #[serde(rename = "page")]
    pub page: Option<u32>,
    #[serde(rename = "perPage")]
    pub per_page: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
    #[serde(rename = "applyRelevance")]
    pub apply_relevance: Option<bool>,
    #[serde(rename = "relevanceThreshold")]
    pub relevance_threshold: Option<f32>,
    #[serde(rename = "fuzzyMatch")]
    pub fuzzy_match: Option<bool>,
    #[serde(rename = "house")]
    pub house: Option<String>,
    pub session: Option<String>,
    #[serde(rename = "parliamentNumber")]
    pub parliament_number: Option<u32>,
    #[serde(rename = "mpId")]
    pub mp_id: Option<u32>,
    #[serde(rename = "fromDate")]
    pub from_date: Option<String>,
    #[serde(rename = "toDate")]
    pub to_date: Option<String>,
    #[serde(rename = "billId")]
    pub bill_id: Option<String>,
    #[serde(rename = "legislationType")]
    pub legislation_type: Option<String>,
    pub title: Option<String>,
    pub year: Option<u32>,
    pub postcode: Option<String>,
    pub limit: Option<u32>,
}
