use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;
use roxmltree::Document;
use serde_json::{Value, json};
use tokio::time::sleep;

use crate::config::AppConfig;
use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use crate::core::http_client::build_http_client;
use crate::features::parliament::dto::{
    FetchBillsArgs, FetchCoreDatasetArgs, FetchLegislationArgs,
};

const CORE_DATASET_BASE: &str = "https://lda.data.parliament.uk";
const MEMBERS_API_BASE: &str = "https://members-api.parliament.uk/api/Members/search";
const BILLS_BASE: &str = "https://bills-api.parliament.uk/api/v1";
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

    pub async fn fetch_core_dataset(&self, args: FetchCoreDatasetArgs) -> Result<Value, AppError> {
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

        let cache_enabled = enable_cache.unwrap_or(true);
        let apply_relevance = apply_relevance.unwrap_or(false);
        let relevance_threshold = relevance_threshold.unwrap_or(self.config.relevance_threshold);
        let fuzzy_match = fuzzy_match.unwrap_or(false);

        match dataset.as_str() {
            "members" | "commonsmembers" | "lordsmembers" => {
                self.fetch_members_dataset(
                    dataset,
                    search_term,
                    page,
                    per_page,
                    cache_enabled,
                    apply_relevance,
                    relevance_threshold,
                    fuzzy_match,
                )
                .await
            }
            _ => {
                self.fetch_legacy_core_dataset(
                    dataset,
                    search_term,
                    page,
                    per_page,
                    cache_enabled,
                    apply_relevance,
                    relevance_threshold,
                    fuzzy_match,
                )
                .await
            }
        }
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

        let search_term = sanitise_optional_text(search_term);
        let house = house
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let session = sanitise_optional_text(session);

        if let Some(ref house_value) = house {
            if !matches!(house_value.as_str(), "commons" | "lords") {
                return Err(AppError::bad_request(format!(
                    "invalid house value: {house_value}"
                )));
            }
        }

        let mut url = Url::parse(&format!("{BILLS_BASE}/Bills"))
            .map_err(|err| AppError::internal(format!("invalid bills url: {err}")))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(term) = &search_term {
                query_pairs.append_pair("searchTerm", term);
            }
            if let Some(house) = &house {
                query_pairs.append_pair("house", house);
            }
            if let Some(session) = &session {
                query_pairs.append_pair("session", session);
            }
            if let Some(parliament_number) = parliament_number {
                query_pairs.append_pair("parliament", &parliament_number.to_string());
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

        self.execute_request(url, cache_key, cache_enabled, ttl)
            .await
    }

    pub async fn fetch_legislation(&self, args: FetchLegislationArgs) -> Result<Value, AppError> {
        if let Some(year) = args.year {
            if year < 1800 {
                return Err(AppError::bad_request(format!(
                    "year must be >= 1800, received {year}"
                )));
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

        let title = sanitise_optional_text(title);
        let legislation_type = legislation_type
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "all".to_string());

        let mut url = Url::parse(&format!("{LEGISLATION_BASE}/{legislation_type}/data.feed"))
            .map_err(|err| AppError::internal(format!("invalid legislation url: {err}")))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(title) = &title {
                query_pairs.append_pair("title", title);
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

        if cache_enabled {
            if let Some(cached) = self.cache.get(&cache_key).await {
                return Ok(cached);
            }
        }

        let mut last_error: Option<AppError> = None;

        for attempt in 0..RETRY_ATTEMPTS {
            let response = self.http_client.get(url.clone()).send().await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let body = resp.text().await.map_err(|err| {
                        AppError::internal(format!("failed to read legislation feed: {err}"))
                    })?;
                    let parsed = parse_legislation_feed(&body)?;

                    if cache_enabled {
                        self.cache
                            .insert(cache_key.clone(), parsed.clone(), ttl)
                            .await;
                    }

                    return Ok(parsed);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "<failed to read body>".to_string());
                    let snippet = text.chars().take(512).collect::<String>();
                    last_error = Some(AppError::upstream_with_data(
                        format!("request to {url} failed with {status}"),
                        json!({
                            "url": url.as_str(),
                            "status": status.as_u16(),
                            "body": snippet,
                        }),
                    ));
                }
                Err(err) => {
                    last_error = Some(AppError::upstream_with_data(
                        format!("network error contacting {url}: {err}"),
                        json!({
                            "url": url.as_str(),
                            "status": Value::Null,
                            "error": err.to_string(),
                        }),
                    ));
                }
            }

            if attempt < RETRY_ATTEMPTS - 1 {
                sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64 + 1))).await;
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::internal("request failed")))
    }

    async fn fetch_members_dataset(
        &self,
        dataset: String,
        search_term: Option<String>,
        page: Option<u32>,
        per_page: Option<u32>,
        cache_enabled: bool,
        apply_relevance: bool,
        relevance_threshold: f32,
        fuzzy_match: bool,
    ) -> Result<Value, AppError> {
        let search_term = sanitise_optional_text(search_term);
        let take = per_page.unwrap_or(20).clamp(1, 100);
        let skip = page.unwrap_or(0).saturating_mul(take);

        let mut url = Url::parse(MEMBERS_API_BASE)
            .map_err(|err| AppError::internal(format!("invalid members api url: {err}")))?;

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(term) = &search_term {
                query_pairs.append_pair("searchText", term);
            }
            query_pairs.append_pair("take", &take.to_string());
            query_pairs.append_pair("skip", &skip.to_string());

            match dataset.as_str() {
                "commonsmembers" => {
                    query_pairs.append_pair("house", "Commons");
                }
                "lordsmembers" => {
                    query_pairs.append_pair("house", "Lords");
                }
                _ => {}
            }
        }

        let cache_key = format!(
            "core_dataset:{}:relevance:{}:threshold:{:.3}:fuzzy:{}",
            url, apply_relevance, relevance_threshold, fuzzy_match
        );
        let ttl = self.config.cache_ttl.members;

        self.execute_request(url, cache_key, cache_enabled, ttl)
            .await
    }

    async fn fetch_legacy_core_dataset(
        &self,
        dataset: String,
        search_term: Option<String>,
        page: Option<u32>,
        per_page: Option<u32>,
        cache_enabled: bool,
        apply_relevance: bool,
        relevance_threshold: f32,
        fuzzy_match: bool,
    ) -> Result<Value, AppError> {
        let search_term = sanitise_optional_text(search_term);

        let mut url = Url::parse(CORE_DATASET_BASE)
            .map_err(|err| AppError::internal(format!("invalid base url: {err}")))?;
        url.set_path(&format!("/{dataset}.json"));

        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(term) = &search_term {
                query_pairs.append_pair("_search", term);
            }
            if let Some(page) = page {
                query_pairs.append_pair("_page", &page.to_string());
            }
            if let Some(per_page) = per_page {
                query_pairs.append_pair("_pageSize", &per_page.to_string());
            }
        }

        let cache_key = format!(
            "core_dataset:{}:relevance:{}:threshold:{:.3}:fuzzy:{}",
            url, apply_relevance, relevance_threshold, fuzzy_match
        );
        let ttl = self.dataset_ttl(&dataset);

        self.execute_request(url, cache_key, cache_enabled, ttl)
            .await
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
                Ok(resp) if resp.status().is_success() => {
                    let json = resp.json::<Value>().await.map_err(|err| {
                        AppError::internal(format!("failed to parse response json: {err}"))
                    })?;

                    if enable_cache {
                        self.cache
                            .insert(cache_key.clone(), json.clone(), ttl)
                            .await;
                    }

                    return Ok(json);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "<failed to read body>".to_string());
                    let body_snippet = text.chars().take(512).collect::<String>();
                    last_error = Some(AppError::upstream_with_data(
                        format!("request to {url} failed with {status}"),
                        json!({
                            "url": url.as_str(),
                            "status": status.as_u16(),
                            "body": body_snippet,
                        }),
                    ));
                }
                Err(err) => {
                    last_error = Some(AppError::upstream_with_data(
                        format!("network error contacting {url}: {err}"),
                        json!({
                            "url": url.as_str(),
                            "status": Value::Null,
                            "error": err.to_string(),
                        }),
                    ));
                }
            }

            if attempt < RETRY_ATTEMPTS - 1 {
                sleep(Duration::from_millis(RETRY_DELAY_MS * (attempt as u64 + 1))).await;
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::internal("request failed")))
    }

    fn dataset_ttl(&self, dataset: &str) -> u64 {
        match dataset {
            "members" | "commonsmembers" | "lordsmembers" => self.config.cache_ttl.members,
            "commonswrittenquestions" | "lordswrittenquestions" | "edms" => {
                self.config.cache_ttl.data
            }
            "commonsdivisions" | "lordsdivisions" => self.config.cache_ttl.data,
            _ => self.config.cache_ttl.data,
        }
    }
}

fn sanitise_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn parse_legislation_feed(feed: &str) -> Result<Value, AppError> {
    let document = Document::parse(feed)
        .map_err(|err| AppError::internal(format!("failed to parse legislation feed: {err}")))?;

    let mut items = Vec::new();

    for entry in document
        .descendants()
        .filter(|node| node.has_tag_name("entry"))
    {
        let title = entry
            .children()
            .find(|node| node.has_tag_name("title"))
            .and_then(|node| node.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_else(|| "Legislation".to_string());

        let uri = entry
            .children()
            .find(|node| node.has_tag_name("id"))
            .and_then(|node| node.text())
            .map(|text| text.trim().to_string());

        let summary = entry
            .children()
            .find(|node| node.has_tag_name("summary"))
            .and_then(|node| node.text())
            .map(|text| text.trim().to_string());

        let year = entry
            .descendants()
            .find(|node| node.has_tag_name("Year"))
            .and_then(|node| node.attribute("Value"))
            .map(|value| value.to_string());

        let doc_type = entry
            .descendants()
            .find(|node| node.has_tag_name("DocumentMainType"))
            .and_then(|node| node.attribute("Value"))
            .map(|value| value.to_string());

        items.push(json!({
            "title": title,
            "year": year,
            "type": doc_type,
            "uri": uri,
            "summary": summary,
        }));

        if items.len() >= 50 {
            // Avoid collecting excessively large responses.
            break;
        }
    }

    let total_results = document
        .descendants()
        .find(|node| node.has_tag_name("totalResults"))
        .and_then(|node| node.text())
        .and_then(|text| text.trim().parse::<usize>().ok());

    let mut payload = json!({ "items": items });
    if let Some(total) = total_results {
        payload["totalResults"] = json!(total);
    }

    Ok(payload)
}
