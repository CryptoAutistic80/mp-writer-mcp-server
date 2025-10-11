use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchCoreDatasetArgs {
    pub dataset: String,
    #[serde(rename = "searchTerm")]
    pub search_term: Option<String>,
    #[serde(rename = "page")]
    pub page: Option<u32>,
    #[serde(rename = "perPage")]
    pub per_page: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
    #[serde(rename = "fuzzyMatch")]
    pub fuzzy_match: Option<bool>,
    #[serde(rename = "applyRelevance")]
    pub apply_relevance: Option<bool>,
    #[serde(rename = "relevanceThreshold")]
    pub relevance_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchBillsArgs {
    #[serde(rename = "searchTerm")]
    pub search_term: Option<String>,
    pub house: Option<String>,
    pub session: Option<String>,
    #[serde(rename = "parliamentNumber")]
    pub parliament_number: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
    #[serde(rename = "applyRelevance")]
    pub apply_relevance: Option<bool>,
    #[serde(rename = "relevanceThreshold")]
    pub relevance_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchHistoricHansardArgs {
    pub house: String,
    pub path: String,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FetchLegislationArgs {
    pub title: Option<String>,
    pub year: Option<u32>,
    #[serde(rename = "type")]
    pub legislation_type: Option<String>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
    #[serde(rename = "applyRelevance")]
    pub apply_relevance: Option<bool>,
    #[serde(rename = "relevanceThreshold")]
    pub relevance_threshold: Option<f32>,
}

