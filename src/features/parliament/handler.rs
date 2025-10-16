use serde_json::Value;

use crate::core::error::AppError;
use crate::features::parliament::client::ParliamentClient;
use crate::features::parliament::dto::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchLegislationArgs,
};

pub async fn handle_fetch_core_dataset(
    client: &ParliamentClient,
    args: FetchCoreDatasetArgs,
) -> Result<Value, AppError> {
    client.fetch_core_dataset(args).await
}

pub async fn handle_fetch_bills(
    client: &ParliamentClient,
    args: FetchBillsArgs,
) -> Result<Value, AppError> {
    client.fetch_bills(args).await
}

pub async fn handle_fetch_legislation(
    client: &ParliamentClient,
    args: FetchLegislationArgs,
) -> Result<Value, AppError> {
    client.fetch_legislation(args).await
}
