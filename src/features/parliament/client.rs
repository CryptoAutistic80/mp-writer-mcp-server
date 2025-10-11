use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;
use serde_json::Value;
use tokio::time::sleep;

use crate::config::AppConfig;
use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use crate::core::http_client::build_http_client;
use crate::features::parliament::dto::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchHistoricHansardArgs, FetchLegislationArgs,
};

const CORE_DATASET_BASE: &str = "https://lda.data.parliament.uk";
const BILLS_BASE: &str = "https://bills-api.parliament.uk/api";
const HANSARD_BASE: &str = "https://api.parliament.uk/historic-hansard";
const LEGISLATION_BASE: &str = "https://www.legislation.gov.uk";
const RETRY_ATTEMPTS: usize = 3;
const RETRY_DELAY_MS: u64 = 500;

pub struct ParliamentClient {
    config: Arc<AppConfig>,
    cache: CacheManager,
    http_client: reqwest::Client,
}

impl ParliamentClient {
    pub fn new(config: Arc<AppConfig>, cache: CacheManager) -> Result<Self, AppError> {
        let http_client = build_http_client(config.disable_proxy)
            .map_err(|err| AppError::internal(format!("failed to build HTTP client: {err}")))?;

        Ok(Self {
            config,
            cache,
            http_client,
        })
    }

    pub async fn fetch_core_dataset(
        &self,
        args: FetchCoreDatasetArgs,
    ) -> Result<Value, AppError> {
        let FetchCoreDatasetArgs {
            dataset,
            search_term,
            page,
            per_page,
            enable_cache,
            fuzzy_match,
            apply_relevance,
            relevance_threshold,
        } = args;

        let mut url = Url::parse(CORE_DATASET_BASE)
            .map_err(|err| AppError::internal(format!("invalid base url: {err}")))?;
        url.set_path(&format!("/{dataset}.json"));

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(search_term) = search_term {
                query_pairs.append_pair("_search", &search_term);
            }
            if let Some(page) = page {
                query_pairs.append_pair("_page", &page.to_string());
            }
            if let Some(per_page) = per_page {
                query_pairs.append_pair("_pageSize", &per_page.to_string());
            }
        }

        let cache_enabled = enable_cache.unwrap_or(true);
        let apply_relevance = apply_relevance.unwrap_or(false);
        let relevance_threshold = relevance_threshold.unwrap_or(self.config.relevance_threshold);
        let fuzzy_match = fuzzy_match.unwrap_or(false);
        let cache_key = format!(
            "core_dataset:{}:relevance:{}:threshold:{:.3}:fuzzy:{}",
            url, apply_relevance, relevance_threshold, fuzzy_match
        );
        let ttl = self.dataset_ttl(&dataset);

        self.execute_request(url, cache_key, cache_enabled, ttl).await
    }

    pub async fn fetch_bills(&self, args: FetchBillsArgs) -> Result<Value, AppError> {
        let FetchBillsArgs {
            search_term,
            house,
            session,
            parliament_number,
            enable_cache,
            apply_relevance,
            relevance_threshold,
        } = args;

        if let Some(ref house_value) = house {
            if !matches!(house_value.as_str(), "commons" | "lords") {
                return Err(AppError::bad_request(format!("invalid house value: {house_value}")));
            }
        }

        let mut url = Url::parse(&format!("{BILLS_BASE}/Bills"))
            .map_err(|err| AppError::internal(format!("invalid bills url: {err}")))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(search_term) = search_term {
                query_pairs.append_pair("SearchTerm", &search_term);
            }
            if let Some(house) = house {
                query_pairs.append_pair("House", &house);
            }
            if let Some(session) = session {
                query_pairs.append_pair("Session", &session);
            }
            if let Some(parliament_number) = parliament_number {
                query_pairs.append_pair("Parliament", &parliament_number.to_string());
            }
        }

        let cache_enabled = enable_cache.unwrap_or(true);
        let apply_relevance = apply_relevance.unwrap_or(false);
        let relevance_threshold = relevance_threshold.unwrap_or(self.config.relevance_threshold);
        let cache_key = format!(
            "bills:{}:relevance:{}:threshold:{:.3}",
            url, apply_relevance, relevance_threshold
        );
        let ttl = self.config.cache_ttl.bills;

        self.execute_request(url, cache_key, cache_enabled, ttl).await
    }

    pub async fn fetch_historic_hansard(
        &self,
        args: FetchHistoricHansardArgs,
    ) -> Result<Value, AppError> {
        if !matches!(args.house.as_str(), "commons" | "lords") {
            return Err(AppError::bad_request(format!("invalid house value: {}", args.house)));
        }

        let encoded_path = args
            .path
            .split('/')
            .map(urlencoding::encode)
            .collect::<Vec<_>>()
            .join("/");

        let url = Url::parse(&format!(
            "{base}/{house}/{path}.json",
            base = HANSARD_BASE,
            house = args.house,
            path = encoded_path
        ))
        .map_err(|err| AppError::internal(format!("invalid hansard url: {err}")))?;

        let cache_enabled = args.enable_cache.unwrap_or(true);
        let cache_key = format!("hansard:{}", url);
        let ttl = self.config.cache_ttl.hansard;

        self.execute_request(url, cache_key, cache_enabled, ttl).await
    }

    pub async fn fetch_legislation(&self, args: FetchLegislationArgs) -> Result<Value, AppError> {
        if let Some(year) = args.year {
            if year < 1800 {
                return Err(AppError::bad_request(format!("year must be >= 1800, received {year}")));
            }
        }

        let FetchLegislationArgs {
            title,
            year,
            legislation_type,
            enable_cache,
            apply_relevance,
            relevance_threshold,
        } = args;

        let legislation_type = legislation_type.unwrap_or_else(|| "all".to_string());
        let mut url = Url::parse(&format!("{LEGISLATION_BASE}/{legislation_type}/data.json"))
            .map_err(|err| AppError::internal(format!("invalid legislation url: {err}")))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(title) = title {
                query_pairs.append_pair("title", &title);
            }
            if let Some(year) = year {
                query_pairs.append_pair("year", &year.to_string());
            }
        }

        let cache_enabled = enable_cache.unwrap_or(true);
        let apply_relevance = apply_relevance.unwrap_or(false);
        let relevance_threshold = relevance_threshold.unwrap_or(self.config.relevance_threshold);
        let cache_key = format!(
            "legislation:{}:relevance:{}:threshold:{:.3}",
            url, apply_relevance, relevance_threshold
        );
        let ttl = self.config.cache_ttl.legislation;

        self.execute_request(url, cache_key, cache_enabled, ttl).await
    }

    async fn execute_request(
        &self,
        url: Url,
        cache_key: String,
        enable_cache: bool,
        ttl: u64,
    ) -> Result<Value, AppError> {
        if enable_cache {
            if let Some(cached) = self.cache.get(&cache_key).await {
                return Ok(cached);
            }
        }

        let mut last_error: Option<AppError> = None;

        for attempt in 0..RETRY_ATTEMPTS {
            let response = self.http_client.get(url.clone()).send().await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let text = resp
                            .text()
                            .await
                            .unwrap_or_else(|_| "<failed to read body>".to_string());
                        last_error = Some(AppError::upstream(format!(
                            "request to {url} failed with {status}: {text}"
                        )));
                    } else {
                        let json = resp
                            .json::<Value>()
                            .await
                            .map_err(|err| AppError::internal(format!(
                                "failed to parse response json: {err}"
                            )))?;

                        if enable_cache {
                            self.cache.insert(cache_key.clone(), json.clone(), ttl).await;
                        }

                        return Ok(json);
                    }
                }
                Err(err) => {
                    last_error = Some(AppError::upstream(format!(
                        "network error contacting {url}: {err}"
                    )));
                }
            }

            if attempt < RETRY_ATTEMPTS - 1 {
                sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64 + 1))).await;
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::internal("request failed".to_string())))
    }

    fn dataset_ttl(&self, dataset: &str) -> u64 {
        match dataset {
            "commonsmembers" | "lordsmembers" => self.config.cache_ttl.members,
            "commonswrittenquestions" | "lordswrittenquestions" | "edms" => self.config.cache_ttl.data,
            "commonsdivisions" | "lordsdivisions" => self.config.cache_ttl.data,
            _ => self.config.cache_ttl.data,
        }
    }
}
