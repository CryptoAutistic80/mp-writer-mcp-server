use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub api_key: String,
    pub disable_proxy: bool,
    pub cache_enabled: bool,
    pub relevance_threshold: f32,
    pub cache_ttl: CacheTtlConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheTtlConfig {
    pub members: u64,
    pub bills: u64,
    pub legislation: u64,
    pub hansard: u64,
    pub data: u64,
}
