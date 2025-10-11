use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheManager {
    enabled: bool,
    capacity: usize,
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

impl CacheManager {
    pub fn new(enabled: bool, capacity: u64) -> Self {
        Self {
            enabled,
            capacity: capacity as usize,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Value> {
        if !self.enabled {
            return None;
        }

        let mut guard = self.store.write().await;
        if let Some(entry) = guard.get(key) {
            if Instant::now() <= entry.expires_at {
                return Some(entry.value.clone());
            }
        }

        guard.remove(key);
        None
    }

    pub async fn insert(&self, key: String, value: Value, ttl_seconds: u64) {
        if !self.enabled {
            return;
        }

        let expires_at = Instant::now() + Duration::from_secs(ttl_seconds);
        let mut guard = self.store.write().await;

        if guard.len() >= self.capacity {
            if let Some(expired_key) = guard
                .iter()
                .filter(|(_, entry)| entry.expires_at <= Instant::now())
                .map(|(k, _)| k.clone())
                .next()
            {
                guard.remove(&expired_key);
            } else if let Some(first_key) = guard.keys().next().cloned() {
                guard.remove(&first_key);
            }
        }

        guard.insert(
            key,
            CacheEntry {
                value,
                expires_at,
            },
        );
    }
}
