mod config;
mod core;
mod features;
mod server;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::load_config;
use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use crate::features::mcp::{McpService, handle_healthcheck, handle_mcp};
use crate::features::parliament::ParliamentClient;
use crate::features::research::ResearchService;
use crate::server::{AppState, require_api_key};

const CACHE_CAPACITY: u64 = 1024;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_tracing();

    let config = Arc::new(load_config()?);
    let cache_manager = CacheManager::new(config.cache_enabled, CACHE_CAPACITY);
    let parliament_client = Arc::new(ParliamentClient::new(config.clone(), cache_manager)?);

    let sled_db = sled::open(&config.db_path).map_err(|err| {
        AppError::internal(format!(
            "failed to open sled database at {}: {err}",
            config.db_path
        ))
    })?;
    let research_tree = sled_db
        .open_tree("research")
        .map_err(|err| AppError::internal(format!("failed to open research cache: {err}")))?;

    let research_data_source: Arc<dyn crate::features::research::ParliamentDataSource> =
        parliament_client.clone();
    let research_service = Arc::new(ResearchService::new(
        config.clone(),
        research_data_source,
        research_tree,
    ));

    let mcp_service = Arc::new(McpService::new(parliament_client, research_service.clone()));
    let app_state = AppState::new(mcp_service.clone(), config.api_key.clone());

    let app = Router::new()
        .route("/api/health", get(handle_healthcheck))
        .route(
            "/api/mcp",
            post(handle_mcp).layer(middleware::from_fn_with_state(
                app_state.clone(),
                require_api_key,
            )),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(%addr, "starting server");
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| AppError::internal(format!("failed to bind: {err}")))?;
    axum::serve(listener, app)
        .await
        .map_err(|err| AppError::internal(format!("server error: {err}")))?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_target(false)
        .init();
}
