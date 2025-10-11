use std::env;

use crate::config::dto::{AppConfig, CacheTtlConfig};
use crate::core::error::AppError;

pub fn load_config() -> Result<AppConfig, AppError> {
    dotenvy::dotenv().ok();

    let port = env::var("MCP_SERVER_PORT")
        .or_else(|_| env::var("DEEP_RESEARCH_MCP_PORT"))
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "4100".to_string())
        .parse::<u16>()
        .map_err(|err| AppError::configuration(format!("invalid port: {err}")))?;

    let api_key = env::var("MCP_API_KEY")
        .or_else(|_| env::var("DEEP_RESEARCH_API_KEY"))
        .map_err(|_| AppError::configuration("MCP_API_KEY is required".to_string()))?;

    let disable_proxy = env::var("MCP_DISABLE_PROXY")
        .ok()
        .or_else(|| env::var("DEEP_RESEARCH_DISABLE_PROXY").ok())
        .map(|value| matches!(value.as_str(), "true" | "1" | "TRUE" | "True"))
        .unwrap_or(false);
    let cache_enabled = parse_bool_env("CACHE_ENABLED", true);

    let relevance_threshold = env::var("RELEVANCE_THRESHOLD")
        .unwrap_or_else(|_| "0.3".to_string())
        .parse::<f32>()
        .map_err(|err| AppError::configuration(format!("invalid RELEVANCE_THRESHOLD: {err}")))?;

    let cache_ttl = CacheTtlConfig {
        members: parse_u64_env("CACHE_TTL_MEMBERS", 3600),
        bills: parse_u64_env("CACHE_TTL_BILLS", 1800),
        legislation: parse_u64_env("CACHE_TTL_LEGISLATION", 7200),
        hansard: parse_u64_env("CACHE_TTL_HANSARD", 3600),
        data: parse_u64_env("CACHE_TTL_DATA", 1800),
    };

    Ok(AppConfig {
        port,
        api_key,
        disable_proxy,
        cache_enabled,
        relevance_threshold,
        cache_ttl,
    })
}

fn parse_bool_env(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|value| matches!(value.as_str(), "true" | "1" | "TRUE" | "True"))
        .unwrap_or(default)
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}
