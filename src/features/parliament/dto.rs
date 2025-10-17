use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct FetchMpActivityArgs {
    #[serde(rename = "mpId")]
    pub mp_id: u32,
    pub limit: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FetchMpVotingRecordArgs {
    #[serde(rename = "mpId")]
    pub mp_id: u32,
    #[serde(rename = "fromDate")]
    pub from_date: Option<String>,
    #[serde(rename = "toDate")]
    pub to_date: Option<String>,
    #[serde(rename = "billId")]
    pub bill_id: Option<String>,
    pub limit: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LookupConstituencyArgs {
    pub postcode: String,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemberInfo {
    pub name_display_as: String,
    pub membership_start_date: Option<String>,
    pub constituency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MpActivityEntry {
    pub id: String,
    pub date: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub title: String,
    pub description: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MpVoteRecord {
    #[serde(rename = "divisionId")]
    pub division_id: Option<String>,
    pub title: Option<String>,
    pub date: Option<String>,
    pub vote: Option<String>,
    pub majority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConstituencyLookupResult {
    #[serde(rename = "constituencyCode")]
    pub constituency_code: Option<String>,
    #[serde(rename = "constituencyName")]
    pub constituency_name: Option<String>,
    #[serde(rename = "mpId")]
    pub mp_id: Option<u32>,
    #[serde(rename = "mpName")]
    pub mp_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchUkLawArgs {
    pub query: String,
    #[serde(rename = "legislationType")]
    pub legislation_type: Option<String>, // "primary", "secondary", "all"
    pub limit: Option<u32>,
    #[serde(rename = "enableCache")]
    pub enable_cache: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UkLawResult {
    pub title: String,
    pub year: Option<String>,
    #[serde(rename = "legislationType")]
    pub legislation_type: String,
    #[serde(rename = "isInForce")]
    pub is_in_force: bool,
    pub url: String,
    pub summary: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}
