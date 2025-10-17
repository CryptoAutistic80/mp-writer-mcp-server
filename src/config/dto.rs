use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub api_key: String,
    pub disable_proxy: bool,
    pub cache_enabled: bool,
    pub relevance_threshold: f32,
    pub cache_ttl: CacheTtlConfig,
    pub db_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheTtlConfig {
    pub members: u64,
    pub bills: u64,
    pub legislation: u64,
    pub data: u64,
    pub research: u64,
    pub activity: u64,
    pub votes: u64,
    pub constituency: u64,
}
