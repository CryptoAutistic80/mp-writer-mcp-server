use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;
use sled::Tree;
use tokio::task;

use crate::core::error::AppError;

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEnvelope<T> {
    stored_at: u64,
    payload: T,
}

pub async fn read_cache<T>(tree: &Tree, key: &str, ttl: u64) -> Result<Option<T>, AppError>
where
    T: DeserializeOwned + Send + 'static,
{
    let tree = tree.clone();
    let key_bytes = key.as_bytes().to_vec();

    task::spawn_blocking(move || -> Result<Option<T>, AppError> {
        let maybe_bytes = tree
            .get(&key_bytes)
            .map_err(|err| AppError::internal(format!("cache lookup failed: {err}")))?;

        if let Some(bytes) = maybe_bytes {
            let envelope: CacheEnvelope<T> = serde_json::from_slice(&bytes).map_err(|err| {
                AppError::internal(format!("failed to decode cached response: {err}"))
            })?;

            if current_timestamp().saturating_sub(envelope.stored_at) <= ttl {
                return Ok(Some(envelope.payload));
            }
        }

        Ok(None)
    })
    .await
    .map_err(|err| AppError::internal(format!("cache task join error: {err}")))?
}

pub async fn write_cache<T>(tree: &Tree, key: &str, value: &T) -> Result<(), AppError>
where
    T: Serialize,
{
    let envelope = CacheEnvelope {
        stored_at: current_timestamp(),
        payload: value,
    };
    let data = serde_json::to_vec(&envelope)
        .map_err(|err| AppError::internal(format!("failed to encode cache entry: {err}")))?;

    let tree_clone = tree.clone();
    let key_bytes = key.as_bytes().to_vec();
    task::spawn_blocking(move || -> Result<(), AppError> {
        tree_clone
            .insert(key_bytes, data)
            .map_err(|err| AppError::internal(format!("failed to write cache entry: {err}")))?;
        Ok(())
    })
    .await
    .map_err(|err| AppError::internal(format!("cache task join error: {err}")))??;

    tree.flush_async()
        .await
        .map_err(|err| AppError::internal(format!("failed to flush cache: {err}")))?;

    Ok(())
}

pub fn normalise_postcode(postcode: &str) -> Option<String> {
    let cleaned = postcode
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    if cleaned.is_empty() {
        return None;
    }

    Some(cleaned.to_uppercase())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
