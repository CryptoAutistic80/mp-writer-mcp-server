use async_trait::async_trait;
use serde_json::Value;

use crate::core::error::AppError;
use crate::features::mcp::dto::{
    FetchToolArgs, FetchToolTarget, SearchToolArgs, SearchToolTarget,
};
use crate::features::parliament::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchLegislationArgs, FetchMpActivityArgs,
    FetchMpVotingRecordArgs, LookupConstituencyArgs, SearchUkLawArgs,
};

#[async_trait]
pub trait ParliamentToolExecutor: Send + Sync {
    async fn search_uk_law(&self, args: SearchUkLawArgs) -> Result<Value, AppError>;
    async fn fetch_bills(&self, args: FetchBillsArgs) -> Result<Value, AppError>;
    async fn fetch_core_dataset(&self, args: FetchCoreDatasetArgs) -> Result<Value, AppError>;
    async fn fetch_legislation(&self, args: FetchLegislationArgs) -> Result<Value, AppError>;
    async fn fetch_mp_activity(&self, args: FetchMpActivityArgs) -> Result<Value, AppError>;
    async fn fetch_mp_voting_record(
        &self,
        args: FetchMpVotingRecordArgs,
    ) -> Result<Value, AppError>;
    async fn lookup_constituency_offline(
        &self,
        args: LookupConstituencyArgs,
    ) -> Result<Value, AppError>;
}

pub async fn handle_search_tool<T>(
    executor: &T,
    args: SearchToolArgs,
) -> Result<Value, AppError>
where
    T: ParliamentToolExecutor + ?Sized,
{
    let SearchToolArgs {
        target,
        query,
        dataset,
        legislation_type,
        limit,
        enable_cache,
        apply_relevance,
        relevance_threshold,
        fuzzy_match,
        house,
        session,
        parliament_number,
        page,
        per_page,
    } = args;

    match target {
        SearchToolTarget::UkLaw => {
            let query = query.ok_or_else(|| {
                AppError::bad_request("search target 'uk_law' requires a query")
            })?;
            let search_args = SearchUkLawArgs {
                query,
                legislation_type,
                limit,
                enable_cache,
            };
            executor.search_uk_law(search_args).await
        }
        SearchToolTarget::Bills => {
            let search_args = FetchBillsArgs {
                search_term: query,
                house,
                session,
                parliament_number,
                enable_cache,
                apply_relevance,
                relevance_threshold,
            };
            executor.fetch_bills(search_args).await
        }
        SearchToolTarget::Dataset => {
            let dataset = dataset.ok_or_else(|| {
                AppError::bad_request("search target 'dataset' requires the dataset field")
            })?;
            let mut search_term = query;
            if let Some(term) = &mut search_term {
                if term.trim().is_empty() {
                    *term = String::new();
                }
            }
            let args = FetchCoreDatasetArgs {
                dataset,
                search_term,
                page,
                per_page,
                enable_cache,
                fuzzy_match,
                apply_relevance,
                relevance_threshold,
            };
            executor.fetch_core_dataset(args).await
        }
    }
}

pub async fn handle_fetch_tool<T>(
    executor: &T,
    args: FetchToolArgs,
) -> Result<Value, AppError>
where
    T: ParliamentToolExecutor + ?Sized,
{
    let FetchToolArgs {
        target,
        dataset,
        search_term,
        page,
        per_page,
        enable_cache,
        apply_relevance,
        relevance_threshold,
        fuzzy_match,
        house,
        session,
        parliament_number,
        mp_id,
        from_date,
        to_date,
        bill_id,
        legislation_type,
        title,
        year,
        postcode,
        limit,
    } = args;

    match target {
        FetchToolTarget::CoreDataset => {
            let dataset = dataset.ok_or_else(|| {
                AppError::bad_request("fetch target 'core_dataset' requires the dataset field")
            })?;
            let args = FetchCoreDatasetArgs {
                dataset,
                search_term,
                page,
                per_page,
                enable_cache,
                fuzzy_match,
                apply_relevance,
                relevance_threshold,
            };
            executor.fetch_core_dataset(args).await
        }
        FetchToolTarget::Bills => {
            let args = FetchBillsArgs {
                search_term,
                house,
                session,
                parliament_number,
                enable_cache,
                apply_relevance,
                relevance_threshold,
            };
            executor.fetch_bills(args).await
        }
        FetchToolTarget::Legislation => {
            let args = FetchLegislationArgs {
                title,
                year,
                legislation_type,
                enable_cache,
                apply_relevance,
                relevance_threshold,
            };
            executor.fetch_legislation(args).await
        }
        FetchToolTarget::MpActivity => {
            let mp_id = mp_id.ok_or_else(|| {
                AppError::bad_request("fetch target 'mp_activity' requires mpId")
            })?;
            let args = FetchMpActivityArgs {
                mp_id,
                limit,
                enable_cache,
            };
            executor.fetch_mp_activity(args).await
        }
        FetchToolTarget::MpVotingRecord => {
            let mp_id = mp_id.ok_or_else(|| {
                AppError::bad_request("fetch target 'mp_voting_record' requires mpId")
            })?;
            let args = FetchMpVotingRecordArgs {
                mp_id,
                from_date,
                to_date,
                bill_id,
                limit,
                enable_cache,
            };
            executor.fetch_mp_voting_record(args).await
        }
        FetchToolTarget::Constituency => {
            let postcode = postcode.ok_or_else(|| {
                AppError::bad_request("fetch target 'constituency' requires postcode")
            })?;
            let args = LookupConstituencyArgs {
                postcode,
                enable_cache,
            };
            executor.lookup_constituency_offline(args).await
        }
    }
}

#[async_trait]
impl ParliamentToolExecutor for crate::features::parliament::ParliamentClient {
    async fn search_uk_law(&self, args: SearchUkLawArgs) -> Result<Value, AppError> {
        crate::features::parliament::handle_search_uk_law(self, args).await
    }

    async fn fetch_bills(&self, args: FetchBillsArgs) -> Result<Value, AppError> {
        crate::features::parliament::handle_fetch_bills(self, args).await
    }

    async fn fetch_core_dataset(&self, args: FetchCoreDatasetArgs) -> Result<Value, AppError> {
        crate::features::parliament::handle_fetch_core_dataset(self, args).await
    }

    async fn fetch_legislation(&self, args: FetchLegislationArgs) -> Result<Value, AppError> {
        crate::features::parliament::handle_fetch_legislation(self, args).await
    }

    async fn fetch_mp_activity(&self, args: FetchMpActivityArgs) -> Result<Value, AppError> {
        crate::features::parliament::handle_fetch_mp_activity(self, args).await
    }

    async fn fetch_mp_voting_record(
        &self,
        args: FetchMpVotingRecordArgs,
    ) -> Result<Value, AppError> {
        crate::features::parliament::handle_fetch_mp_voting_record(self, args).await
    }

    async fn lookup_constituency_offline(
        &self,
        args: LookupConstituencyArgs,
    ) -> Result<Value, AppError> {
        crate::features::parliament::handle_lookup_constituency_offline(self, args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct MockExecutor {
        calls: Arc<Mutex<Vec<String>>>,
        response: Value,
    }

    impl MockExecutor {
        fn with_response(response: Value) -> Self {
            Self {
                response,
                ..Default::default()
            }
        }

        async fn push_call(&self, name: &str) {
            let mut guard = self.calls.lock().await;
            guard.push(name.to_string());
        }
    }

    #[async_trait]
    impl ParliamentToolExecutor for MockExecutor {
        async fn search_uk_law(&self, _args: SearchUkLawArgs) -> Result<Value, AppError> {
            self.push_call("search_uk_law").await;
            Ok(self.response.clone())
        }

        async fn fetch_bills(&self, _args: FetchBillsArgs) -> Result<Value, AppError> {
            self.push_call("fetch_bills").await;
            Ok(self.response.clone())
        }

        async fn fetch_core_dataset(&self, _args: FetchCoreDatasetArgs) -> Result<Value, AppError> {
            self.push_call("fetch_core_dataset").await;
            Ok(self.response.clone())
        }

        async fn fetch_legislation(&self, _args: FetchLegislationArgs) -> Result<Value, AppError> {
            self.push_call("fetch_legislation").await;
            Ok(self.response.clone())
        }

        async fn fetch_mp_activity(&self, _args: FetchMpActivityArgs) -> Result<Value, AppError> {
            self.push_call("fetch_mp_activity").await;
            Ok(self.response.clone())
        }

        async fn fetch_mp_voting_record(
            &self,
            _args: FetchMpVotingRecordArgs,
        ) -> Result<Value, AppError> {
            self.push_call("fetch_mp_voting_record").await;
            Ok(self.response.clone())
        }

        async fn lookup_constituency_offline(
            &self,
            _args: LookupConstituencyArgs,
        ) -> Result<Value, AppError> {
            self.push_call("lookup_constituency_offline").await;
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn routes_uk_law_search() {
        let executor = MockExecutor::with_response(json!({"status": "ok"}));
        let result = handle_search_tool(
            &executor,
            SearchToolArgs {
                target: SearchToolTarget::UkLaw,
                query: Some("climate".to_string()),
                dataset: None,
                legislation_type: Some("primary".to_string()),
                limit: Some(5),
                enable_cache: Some(true),
                apply_relevance: None,
                relevance_threshold: None,
                fuzzy_match: None,
                house: None,
                session: None,
                parliament_number: None,
                page: None,
                per_page: None,
            },
        )
        .await
        .expect("search should succeed");

        assert_eq!(result, json!({"status": "ok"}));
        let calls = executor.calls.lock().await.clone();
        assert_eq!(calls, vec!["search_uk_law"]);
    }

    #[tokio::test]
    async fn routes_dataset_search() {
        let executor = MockExecutor::with_response(json!({"items": []}));
        let _ = handle_search_tool(
            &executor,
            SearchToolArgs {
                target: SearchToolTarget::Dataset,
                query: Some("smith".to_string()),
                dataset: Some("members".to_string()),
                legislation_type: None,
                limit: None,
                enable_cache: Some(false),
                apply_relevance: Some(true),
                relevance_threshold: Some(0.4),
                fuzzy_match: Some(false),
                house: None,
                session: None,
                parliament_number: None,
                page: Some(0),
                per_page: Some(10),
            },
        )
        .await
        .expect("dataset search should succeed");

        let calls = executor.calls.lock().await.clone();
        assert_eq!(calls, vec!["fetch_core_dataset"]);
    }

    #[tokio::test]
    async fn fetch_requires_mp_id() {
        let executor = MockExecutor::default();
        let error = handle_fetch_tool(
            &executor,
            FetchToolArgs {
                target: FetchToolTarget::MpActivity,
                dataset: None,
                search_term: None,
                page: None,
                per_page: None,
                enable_cache: None,
                apply_relevance: None,
                relevance_threshold: None,
                fuzzy_match: None,
                house: None,
                session: None,
                parliament_number: None,
                mp_id: None,
                from_date: None,
                to_date: None,
                bill_id: None,
                legislation_type: None,
                title: None,
                year: None,
                postcode: None,
                limit: Some(5),
            },
        )
        .await
        .expect_err("mp activity should error without mpId");

        match error {
            AppError::BadRequest { message } => {
                assert!(message.contains("mpId"), "unexpected message: {message}");
            }
            other => panic!("expected bad request error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn routes_constituency_fetch() {
        let executor = MockExecutor::with_response(json!({"mp": "Test"}));
        let result = handle_fetch_tool(
            &executor,
            FetchToolArgs {
                target: FetchToolTarget::Constituency,
                dataset: None,
                search_term: None,
                page: None,
                per_page: None,
                enable_cache: Some(true),
                apply_relevance: None,
                relevance_threshold: None,
                fuzzy_match: None,
                house: None,
                session: None,
                parliament_number: None,
                mp_id: None,
                from_date: None,
                to_date: None,
                bill_id: None,
                legislation_type: None,
                title: None,
                year: None,
                postcode: Some("SW1A 1AA".to_string()),
                limit: None,
            },
        )
        .await
        .expect("constituency fetch should succeed");

        assert_eq!(result, json!({"mp": "Test"}));
        let calls = executor.calls.lock().await.clone();
        assert_eq!(calls, vec!["lookup_constituency_offline"]);
    }
}
