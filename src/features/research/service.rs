use std::collections::HashSet;
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
    BillSummaryDto, DebateSummaryDto, LegislationSummaryDto, ResearchRequestDto,
    ResearchResponseDto, StateOfPartiesDto, VoteSummaryDto,
};
use crate::features::research::helpers::{
    DEFAULT_RESULT_LIMIT, build_cache_key, coerce_limit, compose_summary, ensure_keywords,
    expand_search_terms, now_timestamp, parse_bill_results, parse_debate_results,
    parse_legislation_results, parse_state_of_parties, parse_vote_results,
};

#[derive(Serialize, Deserialize)]
struct CachedResearchEntry {
    stored_at: u64,
    payload: ResearchResponseDto,
}

struct CollectionOutcome<T> {
    data: T,
    advisories: Vec<String>,
}

impl<T> CollectionOutcome<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            advisories: Vec::new(),
        }
    }

    fn with_advisories(data: T, advisories: Vec<String>) -> Self {
        Self { data, advisories }
    }
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

        let (bills_outcome, votes_outcome, legislation_outcome, debates_outcome, state_outcome) = tokio::join!(
            bills_future,
            votes_future,
            legislation_future,
            debates_future,
            state_future,
        );

        let CollectionOutcome {
            data: bills,
            advisories: mut bills_notes,
        } = bills_outcome;
        let CollectionOutcome {
            data: votes,
            advisories: mut votes_notes,
        } = votes_outcome;
        let CollectionOutcome {
            data: legislation,
            advisories: mut legislation_notes,
        } = legislation_outcome;
        let CollectionOutcome {
            data: debates,
            advisories: mut debates_notes,
        } = debates_outcome;
        let CollectionOutcome {
            data: state_of_parties,
            advisories: mut state_notes,
        } = state_outcome;

        let mut advisories = Vec::new();
        advisories.append(&mut bills_notes);
        advisories.append(&mut votes_notes);
        advisories.append(&mut legislation_notes);
        advisories.append(&mut debates_notes);
        advisories.append(&mut state_notes);

        if !advisories.is_empty() {
            let mut seen = HashSet::new();
            advisories.retain(|note| seen.insert(note.clone()));
            if advisories.len() > 4 {
                advisories.truncate(4);
            }
        }

        let mut response = ResearchResponseDto {
            summary: String::new(),
            bills,
            debates,
            legislation,
            votes,
            mp_speeches: Vec::new(),
            state_of_parties,
            advisories: Vec::new(),
            cached: false,
        };
        response.summary = compose_summary(topic, &response, &advisories);
        response.advisories = advisories;

        self.store_cache(&cache_key, &response).await?;

        Ok(response)
    }

    async fn collect_bills(
        &self,
        keywords: &[String],
        limit: usize,
    ) -> CollectionOutcome<Vec<BillSummaryDto>> {
        let mut advisories = Vec::new();

        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            let terms = expand_search_terms(keyword);
            for term in terms.iter() {
                for (apply_relevance, threshold, broadened) in [
                    (Some(true), Some(self.config.relevance_threshold), false),
                    (Some(false), Some(0.0_f32), true),
                ] {
                    let args = FetchBillsArgs {
                        search_term: Some(term.clone()),
                        house: None,
                        session: None,
                        parliament_number: None,
                        enable_cache: Some(true),
                        apply_relevance,
                        relevance_threshold: threshold,
                    };

                    match self.data_source.fetch_bills(args).await {
                        Ok(raw) => {
                            let parsed = parse_bill_results(&raw, limit);
                            if !parsed.is_empty() {
                                if broadened || term != keyword {
                                    advisories.push(format!(
                                        "Bills search broadened to \"{term}\" after the initial query returned no results."
                                    ));
                                }
                                return CollectionOutcome::with_advisories(parsed, advisories);
                            }
                        }
                        Err(error) => {
                            warn!(target: "research", %error, term, "failed to fetch bills");
                            advisories.push(format!("Bills lookup for \"{term}\" failed: {error}"));
                        }
                    }
                }
            }

            advisories.push(format!(
                "No bills matched the keyword \"{keyword}\"; try alternative or broader keywords."
            ));
        }

        if advisories.is_empty() {
            advisories.push("Bills service returned no data for this topic.".to_string());
        }

        CollectionOutcome::with_advisories(Vec::new(), advisories)
    }

    async fn collect_votes(
        &self,
        keywords: &[String],
        limit: usize,
    ) -> CollectionOutcome<Vec<VoteSummaryDto>> {
        let mut advisories = Vec::new();

        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            for term in expand_search_terms(keyword).iter() {
                let args = FetchCoreDatasetArgs {
                    dataset: "commonsdivisions".to_string(),
                    search_term: Some(term.clone()),
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
                            if term != keyword {
                                advisories.push(format!(
                                    "Division search broadened to \"{term}\" after the initial keyword returned no results."
                                ));
                            }
                            return CollectionOutcome::with_advisories(parsed, advisories);
                        }
                    }
                    Err(error) => {
                        warn!(target: "research", %error, term, "failed to fetch divisions");
                        advisories.push(format!("Division lookup for \"{term}\" failed: {error}"));
                    }
                }
            }

            advisories.push(format!(
                "No Commons divisions matched the keyword \"{keyword}\"; consider broader vote terms."
            ));
        }

        if advisories.is_empty() {
            advisories.push("No Commons divisions were retrieved for this topic.".to_string());
        }

        CollectionOutcome::with_advisories(Vec::new(), advisories)
    }

    async fn collect_legislation(
        &self,
        keywords: &[String],
        limit: usize,
    ) -> CollectionOutcome<Vec<LegislationSummaryDto>> {
        let mut advisories = Vec::new();

        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            for term in expand_search_terms(keyword).iter() {
                for (apply_relevance, threshold, broadened) in [
                    (Some(true), Some(self.config.relevance_threshold), false),
                    (Some(false), Some(0.0_f32), true),
                ] {
                    let args = FetchLegislationArgs {
                        title: Some(term.clone()),
                        year: None,
                        legislation_type: None,
                        enable_cache: Some(true),
                        apply_relevance,
                        relevance_threshold: threshold,
                    };

                    match self.data_source.fetch_legislation(args).await {
                        Ok(raw) => {
                            let parsed = parse_legislation_results(&raw, limit);
                            if !parsed.is_empty() {
                                if broadened || term != keyword {
                                    advisories.push(format!(
                                        "Legislation search broadened to \"{term}\" after the initial keyword returned no results."
                                    ));
                                }
                                return CollectionOutcome::with_advisories(parsed, advisories);
                            }
                        }
                        Err(error) => {
                            warn!(target: "research", %error, term, "failed to fetch legislation");
                            advisories
                                .push(format!("Legislation lookup for \"{term}\" failed: {error}"));
                        }
                    }
                }
            }

            advisories.push(format!(
                "No legislation matched the keyword \"{keyword}\"; try alternate titles or verify the act year."
            ));
        }

        if advisories.is_empty() {
            advisories.push("Legislation search produced no matches for this topic.".to_string());
        }

        CollectionOutcome::with_advisories(Vec::new(), advisories)
    }

    async fn collect_debates(
        &self,
        keywords: &[String],
        limit: usize,
    ) -> CollectionOutcome<Vec<DebateSummaryDto>> {
        let mut advisories = Vec::new();

        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            for term in expand_search_terms(keyword).iter() {
                let args = FetchCoreDatasetArgs {
                    dataset: "commonsdebates".to_string(),
                    search_term: Some(term.clone()),
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
                            if term != keyword {
                                advisories.push(format!(
                                    "Debate search broadened to \"{term}\" after the initial keyword returned no results."
                                ));
                            }
                            return CollectionOutcome::with_advisories(parsed, advisories);
                        }
                    }
                    Err(error) => {
                        warn!(target: "research", %error, term, "failed to fetch debates");
                        advisories.push(format!("Debate lookup for \"{term}\" failed: {error}"));
                    }
                }
            }

            advisories.push(format!(
                "No Commons debates matched the keyword \"{keyword}\"; try broader debate topics or different dates."
            ));
        }

        if advisories.is_empty() {
            advisories.push("Debate search returned no results for this topic.".to_string());
        }

        CollectionOutcome::with_advisories(Vec::new(), advisories)
    }

    async fn collect_state_of_parties(
        &self,
        include: bool,
    ) -> CollectionOutcome<Option<StateOfPartiesDto>> {
        if !include {
            return CollectionOutcome::new(None);
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

        let mut advisories = Vec::new();

        let data = match self.data_source.fetch_core_dataset(args).await {
            Ok(raw) => parse_state_of_parties(&raw),
            Err(error) => {
                warn!(target: "research", %error, "failed to fetch state of parties data");
                advisories.push(
                    "State of parties data is temporarily unavailable; seat counts were omitted."
                        .to_string(),
                );
                None
            }
        };

        CollectionOutcome::with_advisories(data, advisories)
    }

    async fn try_get_cached(&self, key: &str) -> Result<Option<ResearchResponseDto>, AppError> {
        let tree = self.cache_tree.clone();
        let key_bytes = key.as_bytes().to_vec();
        let ttl = self.cache_ttl;

        let result =
            task::spawn_blocking(move || -> Result<Option<ResearchResponseDto>, AppError> {
                let maybe_bytes = tree
                    .get(&key_bytes)
                    .map_err(|err| AppError::internal(format!("cache lookup failed: {err}")))?;

                if let Some(bytes) = maybe_bytes {
                    let entry: CachedResearchEntry =
                        serde_json::from_slice(&bytes).map_err(|err| {
                            AppError::internal(format!(
                                "failed to decode cached research entry: {err}"
                            ))
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
        let data = serde_json::to_vec(&entry).map_err(|err| {
            AppError::internal(format!("failed to serialise research cache entry: {err}"))
        })?;

        let tree = self.cache_tree.clone();
        let key_bytes = key.as_bytes().to_vec();
        task::spawn_blocking(move || -> Result<(), AppError> {
            tree.insert(key_bytes, data).map_err(|err| {
                AppError::internal(format!("failed to persist research cache entry: {err}"))
            })?;
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
