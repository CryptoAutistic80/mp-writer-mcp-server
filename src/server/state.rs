use std::sync::Arc;

use crate::features::mcp::McpService;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<McpService>,
    pub api_key: Arc<String>,
}

impl AppState {
    pub fn new(service: Arc<McpService>, api_key: String) -> Self {
        Self {
            service,
            api_key: Arc::new(api_key),
        }
    }
}
