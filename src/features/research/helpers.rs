use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::features::research::dto::{
    BillSummaryDto, DebateSummaryDto, LegislationSummaryDto, PartyBreakdownDto, ResearchRequestDto,
    ResearchResponseDto, StateOfPartiesDto, VoteSummaryDto,
};

pub(super) const DEFAULT_RESULT_LIMIT: usize = 5;
pub(super) const MAX_RESULT_LIMIT: usize = 10;

pub(super) fn coerce_limit(limit: Option<usize>) -> usize {
    limit
        .filter(|value| *value > 0)
        .map(|value| value.min(MAX_RESULT_LIMIT))
        .unwrap_or(DEFAULT_RESULT_LIMIT)
}

pub(super) fn ensure_keywords(topic: &str, explicit: &[String]) -> Vec<String> {
    let mut keywords: Vec<String> = explicit
        .iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_lowercase())
            }
        })
        .collect();

    let topic_value = topic.trim().to_lowercase();
    if !topic_value.is_empty() && !keywords.iter().any(|value| value == &topic_value) {
        keywords.insert(0, topic_value);
    }

    keywords
}

pub(super) fn expand_search_terms(keyword: &str) -> Vec<String> {
    let trimmed = keyword.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut terms = Vec::new();
    push_unique(&mut terms, trimmed);

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if let Some(first) = parts.first() {
        push_unique(&mut terms, first);
    }
    if let Some(last) = parts.last() {
        push_unique(&mut terms, last);
    }

    terms
}

fn push_unique(terms: &mut Vec<String>, value: &str) {
    let candidate = value.trim().to_lowercase();
    if candidate.len() < 3 {
        return;
    }
    if !terms.iter().any(|existing| existing == &candidate) {
        terms.push(candidate);
    }
}

pub(super) fn build_cache_key(request: &ResearchRequestDto) -> String {
    let mut bill_keywords = request
        .bill_keywords
        .iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_lowercase())
            }
        })
        .collect::<Vec<_>>();
    bill_keywords.sort();

    let mut debate_keywords = request
        .debate_keywords
        .iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_lowercase())
            }
        })
        .collect::<Vec<_>>();
    debate_keywords.sort();

    format!(
        "topic:{}|bills:{}|debates:{}|mp:{}|state:{}|limit:{}",
        request.topic.trim().to_lowercase(),
        bill_keywords.join(","),
        debate_keywords.join(","),
        request
            .mp_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        request.include_state_of_parties,
        request.limit.unwrap_or(DEFAULT_RESULT_LIMIT)
    )
}

pub(super) fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(super) fn parse_bill_results(value: &Value, limit: usize) -> Vec<BillSummaryDto> {
    let mut results = Vec::new();
    if let Some(items) = locate_array(value, &["items", "results", "bills"]) {
        for item in items.iter() {
            let title = first_string(item, &["title", "shortTitle", "name"]).unwrap_or_else(|| {
                first_string(item, &["billName", "officialTitle"])
                    .unwrap_or_else(|| "Unknown bill".to_string())
            });

            let stage = find_value(item, "billStage")
                .and_then(|stage| first_string(stage, &["description", "name"]))
                .or_else(|| first_string(item, &["stage", "currentStage"]));

            let last_update = first_string(item, &["lastUpdate", "lastUpdated", "updated"]);

            let link = find_value(item, "billId")
                .and_then(|id| id.as_i64())
                .map(|id| format!("https://bills.parliament.uk/bills/{id}"))
                .or_else(|| first_string(item, &["link", "uri", "url"]));

            results.push(BillSummaryDto {
                title,
                stage,
                last_update,
                link,
            });

            if results.len() >= limit {
                break;
            }
        }
    }

    results
}

pub(super) fn parse_legislation_results(value: &Value, limit: usize) -> Vec<LegislationSummaryDto> {
    let mut results = Vec::new();
    if let Some(items) = locate_array(value, &["legislation", "results", "items"]) {
        for item in items.iter() {
            let title = first_string(item, &["title", "name", "titleXml"])
                .unwrap_or_else(|| "Legislation".to_string());
            let year = first_string(item, &["year", "Year"]);
            let legislation_type = first_string(item, &["type", "Type", "legislationType"]);
            let uri = first_string(item, &["uri", "URI", "_about"]);

            results.push(LegislationSummaryDto {
                title,
                year,
                legislation_type,
                uri,
            });

            if results.len() >= limit {
                break;
            }
        }
    }

    results
}

pub(super) fn parse_vote_results(value: &Value, limit: usize) -> Vec<VoteSummaryDto> {
    let mut results = Vec::new();
    if let Some(items) = locate_array(value, &["items", "results", "votes"]) {
        for item in items.iter() {
            let title = first_string(item, &["title", "Title", "motion"])
                .unwrap_or_else(|| "Division".to_string());
            let division_number = first_string(item, &["divisionNumber", "DivisionNumber"]);
            let date = first_string(item, &["date", "Date"]);
            let result_value = first_string(item, &["result", "Result"]);
            let ayes = first_integer(item, &["ayes", "Ayes", "ayesCount"]);
            let noes = first_integer(item, &["noes", "Noes", "noesCount"]);
            let link = first_string(item, &["uri", "_about", "link"]);

            results.push(VoteSummaryDto {
                division_number,
                title,
                date,
                ayes,
                noes,
                result: result_value,
                link,
            });

            if results.len() >= limit {
                break;
            }
        }
    }

    results
}

pub(super) fn parse_debate_results(value: &Value, limit: usize) -> Vec<DebateSummaryDto> {
    let mut results = Vec::new();
    if let Some(items) = locate_array(value, &["items", "results", "debates"]) {
        for item in items.iter() {
            let title = first_string(item, &["title", "Title", "subject"])
                .unwrap_or_else(|| "Debate".to_string());
            let house = first_string(item, &["house", "House"]);
            let date = first_string(item, &["date", "Date"]);
            let link = first_string(item, &["uri", "_about", "link"]);
            let highlight = first_string(item, &["summary", "Synopsis", "description"])
                .or_else(|| first_string(item, &["excerpt"]))
                .map(truncate_summary);

            results.push(DebateSummaryDto {
                title,
                house,
                date,
                link,
                highlight,
            });

            if results.len() >= limit {
                break;
            }
        }
    }

    results
}

pub(super) fn parse_state_of_parties(value: &Value) -> Option<StateOfPartiesDto> {
    let mut parties = Vec::new();
    if let Some(items) = locate_array(value, &["items", "results", "parties"]) {
        for item in items.iter() {
            let name = first_string(item, &["party", "name", "Party"])
                .unwrap_or_else(|| "Unknown".to_string());
            let seats = first_integer(item, &["seats", "Seats", "memberCount"]);

            parties.push(PartyBreakdownDto { name, seats });
        }
    }

    if parties.is_empty() {
        return None;
    }

    let total_seats = first_integer(value, &["totalSeats", "TotalSeats", "total"]);
    let last_updated = first_string(value, &["lastUpdated", "LastUpdated", "date"]);

    Some(StateOfPartiesDto {
        total_seats,
        last_updated,
        parties,
    })
}

pub(super) fn compose_summary(
    topic: &str,
    response: &ResearchResponseDto,
    advisories: &[String],
) -> String {
    let mut segments = Vec::new();

    if let Some(bill) = response.bills.first() {
        let mut detail = bill.title.clone();
        if let Some(stage) = &bill.stage {
            detail.push_str(&format!(" (current stage: {stage})"));
        }
        segments.push(format!("Priority bill: {detail}"));
    }

    if let Some(legislation) = response.legislation.first() {
        segments.push(format!(
            "Relevant legislation: {}{}",
            legislation.title,
            legislation
                .year
                .as_ref()
                .map(|value| format!(" ({value})"))
                .unwrap_or_default()
        ));
    }

    if let Some(vote) = response.votes.first() {
        let mut detail = vote.title.clone();
        if let Some(result) = &vote.result {
            detail.push_str(&format!(" ({result})"));
        }
        segments.push(format!("Recent division: {detail}"));
    }

    if let Some(debate) = response.debates.first() {
        let mut detail = debate.title.clone();
        if let Some(date) = &debate.date {
            detail.push_str(&format!(" ({date})"));
        }
        segments.push(format!("Debate highlight: {detail}"));
    }

    if let Some(state) = &response.state_of_parties {
        if !state.parties.is_empty() {
            let top_party = &state.parties[0];
            segments.push(format!(
                "House balance: {} holding {:?} seats",
                top_party.name, top_party.seats
            ));
        }
    }

    if segments.is_empty() {
        segments.push(
            "No authoritative parliamentary sources were retrieved; consider broadening the topic keywords.".to_string()
        );
    }

    for note in advisories.iter().take(3) {
        segments.push(format!("Note: {note}"));
    }

    let mut summary = format!("Key research findings on \"{}\":", topic.trim());
    for segment in segments {
        summary.push('\n');
        summary.push_str("- ");
        summary.push_str(&segment);
    }

    summary
}

fn truncate_summary(value: String) -> String {
    const MAX_LEN: usize = 220;
    if value.len() <= MAX_LEN {
        return value;
    }

    let mut truncated = value.chars().take(MAX_LEN).collect::<String>();
    truncated.push_str("…");
    truncated
}

fn locate_array<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    for key in keys {
        if let Some(array) = find_value(value, key).and_then(|inner| inner.as_array()) {
            if !array.is_empty() {
                return Some(array);
            }
        }
    }
    None
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(entry) = find_value(value, key) {
            if let Some(text) = entry.as_str() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            } else if entry.is_object() {
                if let Some(text) = first_string(entry, &["text", "value", "description"]) {
                    return Some(text);
                }
            }
        }
    }
    None
}

fn first_integer(value: &Value, keys: &[&str]) -> Option<i64> {
    for key in keys {
        if let Some(entry) = find_value(value, key) {
            if let Some(number) = entry.as_i64() {
                return Some(number);
            }
            if let Some(text) = entry.as_str() {
                if let Ok(parsed) = text.trim().parse::<i64>() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

fn find_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_object().and_then(|object| {
        object
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    })
}
