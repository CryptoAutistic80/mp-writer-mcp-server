use serde_json::Value;

use crate::core::error::AppError;
use crate::features::parliament::client::ParliamentClient;
use crate::features::parliament::dto::{
    ConstituencyLookupResult, FetchBillsArgs, FetchCoreDatasetArgs, FetchLegislationArgs,
    FetchMpActivityArgs, FetchMpVotingRecordArgs, LookupConstituencyArgs, SearchUkLawArgs,
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

pub async fn handle_fetch_mp_activity(
    client: &ParliamentClient,
    args: FetchMpActivityArgs,
) -> Result<Value, AppError> {
    let activities = client.fetch_mp_activity(args).await?;
    serde_json::to_value(activities)
        .map_err(|err| AppError::internal(format!("failed to serialise activities: {err}")))
}

pub async fn handle_fetch_mp_voting_record(
    client: &ParliamentClient,
    args: FetchMpVotingRecordArgs,
) -> Result<Value, AppError> {
    let votes = client.fetch_mp_voting_record(args).await?;
    serde_json::to_value(votes)
        .map_err(|err| AppError::internal(format!("failed to serialise votes: {err}")))
}

pub async fn handle_lookup_constituency_offline(
    client: &ParliamentClient,
    args: LookupConstituencyArgs,
) -> Result<Value, AppError> {
    let result: ConstituencyLookupResult = client.lookup_constituency_offline(args).await?;
    serde_json::to_value(result).map_err(|err| {
        AppError::internal(format!(
            "failed to serialise constituency lookup response: {err}"
        ))
    })
}

pub async fn handle_search_uk_law(
    client: &ParliamentClient,
    args: SearchUkLawArgs,
) -> Result<Value, AppError> {
    let results = client.search_uk_law(args).await?;
    serde_json::to_value(results).map_err(|err| {
        AppError::internal(format!("failed to serialise UK law search results: {err}"))
    })
}
