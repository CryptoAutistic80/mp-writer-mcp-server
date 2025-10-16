use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ResearchRequestDto {
    pub topic: String,
    #[serde(default)]
    pub bill_keywords: Vec<String>,
    #[serde(default)]
    pub debate_keywords: Vec<String>,
    pub mp_id: Option<u32>,
    #[serde(default)]
    pub include_state_of_parties: bool,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResponseDto {
    pub summary: String,
    pub bills: Vec<BillSummaryDto>,
    pub debates: Vec<DebateSummaryDto>,
    pub legislation: Vec<LegislationSummaryDto>,
    pub votes: Vec<VoteSummaryDto>,
    pub mp_speeches: Vec<SpeechSummaryDto>,
    pub state_of_parties: Option<StateOfPartiesDto>,
    #[serde(default)]
    pub advisories: Vec<String>,
    #[serde(default)]
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillSummaryDto {
    pub title: String,
    pub stage: Option<String>,
    pub last_update: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateSummaryDto {
    pub title: String,
    pub house: Option<String>,
    pub date: Option<String>,
    pub link: Option<String>,
    pub highlight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegislationSummaryDto {
    pub title: String,
    pub year: Option<String>,
    #[serde(rename = "type")]
    pub legislation_type: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteSummaryDto {
    pub division_number: Option<String>,
    pub title: String,
    pub date: Option<String>,
    pub ayes: Option<i64>,
    pub noes: Option<i64>,
    pub result: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSummaryDto {
    pub member_name: Option<String>,
    pub date: Option<String>,
    pub excerpt: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateOfPartiesDto {
    pub total_seats: Option<i64>,
    pub last_updated: Option<String>,
    pub parties: Vec<PartyBreakdownDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyBreakdownDto {
    pub name: String,
    pub seats: Option<i64>,
}
