use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Tree;
use tokio::task;
use tracing::warn;

use crate::config::AppConfig;
use crate::core::error::AppError;
use crate::features::parliament::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchLegislationArgs, ParliamentClient,
};
use crate::features::research::dto::{
    BillSummaryDto, DebateSummaryDto, LegislationSummaryDto, ResearchRequestDto, ResearchResponseDto,
    StateOfPartiesDto, VoteSummaryDto,
};
use crate::features::research::helpers::{
    build_cache_key, coerce_limit, compose_summary, ensure_keywords, now_timestamp,
    parse_bill_results, parse_debate_results, parse_legislation_results, parse_state_of_parties,
    parse_vote_results, DEFAULT_RESULT_LIMIT,
};

#[derive(Serialize, Deserialize)]
struct CachedResearchEntry {
    stored_at: u64,
    payload: ResearchResponseDto,
}

pub struct ResearchService {
    config: Arc<AppConfig>,
    data_source: Arc<dyn ParliamentDataSource>,
    cache_tree: Tree,
    cache_ttl: u64,
}

#[async_trait]
pub trait ParliamentDataSource: Send + Sync {
    async fn fetch_bills(&self, args: FetchBillsArgs) -> Result<Value, AppError>;
    async fn fetch_core_dataset(&self, args: FetchCoreDatasetArgs) -> Result<Value, AppError>;
    async fn fetch_legislation(&self, args: FetchLegislationArgs) -> Result<Value, AppError>;
}

#[async_trait]
impl ParliamentDataSource for ParliamentClient {
    async fn fetch_bills(&self, args: FetchBillsArgs) -> Result<Value, AppError> {
        ParliamentClient::fetch_bills(self, args).await
    }

    async fn fetch_core_dataset(&self, args: FetchCoreDatasetArgs) -> Result<Value, AppError> {
        ParliamentClient::fetch_core_dataset(self, args).await
    }

    async fn fetch_legislation(&self, args: FetchLegislationArgs) -> Result<Value, AppError> {
        ParliamentClient::fetch_legislation(self, args).await
    }
}

impl ResearchService {
    pub fn new(
        config: Arc<AppConfig>,
        data_source: Arc<dyn ParliamentDataSource>,
        cache_tree: Tree,
    ) -> Self {
        Self {
            cache_ttl: config.cache_ttl.research,
            config,
            data_source,
            cache_tree,
        }
    }

    pub async fn run_research(
        &self,
        request: ResearchRequestDto,
    ) -> Result<ResearchResponseDto, AppError> {
        let topic = request.topic.trim();
        if topic.is_empty() {
            return Err(AppError::bad_request("topic must not be empty".to_string()));
        }

        let cache_key = build_cache_key(&request);
        if let Some(mut cached) = self.try_get_cached(&cache_key).await? {
            cached.cached = true;
            return Ok(cached);
        }

        let bill_keywords = ensure_keywords(topic, &request.bill_keywords);
        let debate_keywords = ensure_keywords(topic, &request.debate_keywords);
        let limit = coerce_limit(request.limit);

        let bills_future = self.collect_bills(&bill_keywords, limit);
        let votes_future = self.collect_votes(&bill_keywords, limit);
        let legislation_future = self.collect_legislation(&bill_keywords, limit);
        let debates_future = self.collect_debates(&debate_keywords, limit);
        let state_future = self.collect_state_of_parties(request.include_state_of_parties);

        let (bills, votes, legislation, debates, state_of_parties) = tokio::join!(
            bills_future,
            votes_future,
            legislation_future,
            debates_future,
            state_future,
        );

        let mut response = ResearchResponseDto {
            summary: String::new(),
            bills,
            debates,
            legislation,
            votes,
            mp_speeches: Vec::new(),
            state_of_parties,
            cached: false,
        };
        response.summary = compose_summary(topic, &response);

        self.store_cache(&cache_key, &response).await?;

        Ok(response)
    }

    async fn collect_bills(&self, keywords: &[String], limit: usize) -> Vec<BillSummaryDto> {
        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            let args = FetchBillsArgs {
                search_term: Some(keyword.clone()),
                house: None,
                session: None,
                parliament_number: None,
                enable_cache: Some(true),
                apply_relevance: Some(true),
                relevance_threshold: Some(self.config.relevance_threshold),
            };

            match self.data_source.fetch_bills(args).await {
                Ok(raw) => {
                    let parsed = parse_bill_results(&raw, limit);
                    if !parsed.is_empty() {
                        return parsed;
                    }
                }
                Err(error) => {
                    warn!(target: "research", %error, keyword, "failed to fetch bills");
                }
            }
        }

        Vec::new()
    }

    async fn collect_votes(&self, keywords: &[String], limit: usize) -> Vec<VoteSummaryDto> {
        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            let args = FetchCoreDatasetArgs {
                dataset: "commonsdivisions".to_string(),
                search_term: Some(keyword.clone()),
                page: Some(0),
                per_page: Some(limit as u32),
                enable_cache: Some(true),
                fuzzy_match: Some(true),
                apply_relevance: Some(true),
                relevance_threshold: Some(self.config.relevance_threshold),
            };

            match self.data_source.fetch_core_dataset(args).await {
                Ok(raw) => {
                    let parsed = parse_vote_results(&raw, limit);
                    if !parsed.is_empty() {
                        return parsed;
                    }
                }
                Err(error) => {
                    warn!(target: "research", %error, keyword, "failed to fetch divisions");
                }
            }
        }

        Vec::new()
    }

    async fn collect_legislation(
        &self,
        keywords: &[String],
        limit: usize,
    ) -> Vec<LegislationSummaryDto> {
        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            let args = FetchLegislationArgs {
                title: Some(keyword.clone()),
                year: None,
                legislation_type: None,
                enable_cache: Some(true),
                apply_relevance: Some(true),
                relevance_threshold: Some(self.config.relevance_threshold),
            };

            match self.data_source.fetch_legislation(args).await {
                Ok(raw) => {
                    let parsed = parse_legislation_results(&raw, limit);
                    if !parsed.is_empty() {
                        return parsed;
                    }
                }
                Err(error) => {
                    warn!(target: "research", %error, keyword, "failed to fetch legislation");
                }
            }
        }

        Vec::new()
    }

    async fn collect_debates(&self, keywords: &[String], limit: usize) -> Vec<DebateSummaryDto> {
        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            let args = FetchCoreDatasetArgs {
                dataset: "commonsdebates".to_string(),
                search_term: Some(keyword.clone()),
                page: Some(0),
                per_page: Some(limit as u32),
                enable_cache: Some(true),
                fuzzy_match: Some(true),
                apply_relevance: Some(true),
                relevance_threshold: Some(self.config.relevance_threshold),
            };

            match self.data_source.fetch_core_dataset(args).await {
                Ok(raw) => {
                    let parsed = parse_debate_results(&raw, limit);
                    if !parsed.is_empty() {
                        return parsed;
                    }
                }
                Err(error) => {
                    warn!(target: "research", %error, keyword, "failed to fetch debates");
                }
            }
        }

        Vec::new()
    }

    async fn collect_state_of_parties(&self, include: bool) -> Option<StateOfPartiesDto> {
        if !include {
            return None;
        }

        let args = FetchCoreDatasetArgs {
            dataset: "stateofparties".to_string(),
            search_term: None,
            page: None,
            per_page: Some(DEFAULT_RESULT_LIMIT as u32),
            enable_cache: Some(true),
            fuzzy_match: Some(false),
            apply_relevance: Some(false),
            relevance_threshold: Some(self.config.relevance_threshold),
        };

        match self.data_source.fetch_core_dataset(args).await {
            Ok(raw) => parse_state_of_parties(&raw),
            Err(error) => {
                warn!(target: "research", %error, "failed to fetch state of parties data");
                None
            }
        }
    }

    async fn try_get_cached(&self, key: &str) -> Result<Option<ResearchResponseDto>, AppError> {
        let tree = self.cache_tree.clone();
        let key_bytes = key.as_bytes().to_vec();
        let ttl = self.cache_ttl;

        let result = task::spawn_blocking(move || -> Result<Option<ResearchResponseDto>, AppError> {
            let maybe_bytes = tree
                .get(&key_bytes)
                .map_err(|err| AppError::internal(format!("cache lookup failed: {err}")))?;

            if let Some(bytes) = maybe_bytes {
                let entry: CachedResearchEntry = serde_json::from_slice(&bytes).map_err(|err| {
                    AppError::internal(format!("failed to decode cached research entry: {err}"))
                })?;
                if now_timestamp().saturating_sub(entry.stored_at) <= ttl {
                    return Ok(Some(entry.payload));
                }
            }

            Ok(None)
        })
        .await
        .map_err(|err| AppError::internal(format!("cache task join error: {err}")))?;

        result
    }

    async fn store_cache(&self, key: &str, response: &ResearchResponseDto) -> Result<(), AppError> {
        let mut cacheable = response.clone();
        cacheable.cached = false;
        let entry = CachedResearchEntry {
            stored_at: now_timestamp(),
            payload: cacheable,
        };
        let data = serde_json::to_vec(&entry)
            .map_err(|err| AppError::internal(format!("failed to serialise research cache entry: {err}")))?;

        let tree = self.cache_tree.clone();
        let key_bytes = key.as_bytes().to_vec();
        task::spawn_blocking(move || -> Result<(), AppError> {
            tree.insert(key_bytes, data)
                .map_err(|err| AppError::internal(format!("failed to persist research cache entry: {err}")))?;
            Ok(())
        })
        .await
        .map_err(|err| AppError::internal(format!("cache task join error: {err}")))??;

        self.cache_tree
            .flush_async()
            .await
            .map_err(|err| AppError::internal(format!("failed to flush research cache: {err}")))?;

        Ok(())
    }
}
