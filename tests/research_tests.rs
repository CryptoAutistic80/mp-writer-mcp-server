use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use tokio::sync::Mutex;

use mp_writer_mcp_server::config::{AppConfig, CacheTtlConfig};
use mp_writer_mcp_server::core::error::AppError;
use mp_writer_mcp_server::features::research::{
    ParliamentDataSource, ResearchRequestDto, ResearchService,
};

struct MockParliamentDataSource {
    bills: serde_json::Value,
    divisions: serde_json::Value,
    legislation: serde_json::Value,
    debates: serde_json::Value,
    parties: serde_json::Value,
    calls: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockParliamentDataSource {
    fn new() -> Self {
        Self {
            bills: json!({
                "items": [
                    {
                        "title": "Climate Change Bill",
                        "billStage": {"description": "Committee"},
                        "lastUpdate": "2024-06-01",
                        "billId": 123
                    }
                ]
            }),
            divisions: json!({
                "items": [
                    {
                        "title": "Division on Climate",
                        "divisionNumber": "12",
                        "date": "2024-05-20",
                        "ayes": 300,
                        "noes": 200,
                        "result": "Ayes",
                        "uri": "https://example.com/division/12"
                    }
                ]
            }),
            legislation: json!({
                "items": [
                    {
                        "title": "Climate Act",
                        "year": "2008",
                        "type": "ukpga",
                        "uri": "https://www.legislation.gov.uk/id/ukpga/2008/27"
                    }
                ]
            }),
            debates: json!({
                "items": [
                    {
                        "title": "Climate debate",
                        "house": "Commons",
                        "date": "2024-05-10",
                        "summary": "Members discussed climate action.",
                        "uri": "https://example.com/debate"
                    }
                ]
            }),
            parties: json!({
                "items": [
                    {"party": "Example Party", "seats": 300}
                ],
                "totalSeats": 650,
                "lastUpdated": "2024-06-01"
            }),
            calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn record_call(&self, key: &str) {
        let mut guard = self.calls.lock().await;
        *guard.entry(key.to_string()).or_insert(0) += 1;
    }

    async fn count_for(&self, key: &str) -> usize {
        let guard = self.calls.lock().await;
        guard.get(key).copied().unwrap_or(0)
    }
}

#[async_trait]
impl ParliamentDataSource for MockParliamentDataSource {
    async fn fetch_bills(
        &self,
        _args: mp_writer_mcp_server::features::parliament::FetchBillsArgs,
    ) -> Result<serde_json::Value, AppError> {
        self.record_call("bills").await;
        Ok(self.bills.clone())
    }

    async fn fetch_core_dataset(
        &self,
        args: mp_writer_mcp_server::features::parliament::FetchCoreDatasetArgs,
    ) -> Result<serde_json::Value, AppError> {
        self.record_call(&args.dataset).await;
        match args.dataset.as_str() {
            "commonsdivisions" => Ok(self.divisions.clone()),
            "commonsdebates" => Ok(self.debates.clone()),
            "stateofparties" => Ok(self.parties.clone()),
            _ => Ok(serde_json::Value::Null),
        }
    }

    async fn fetch_legislation(
        &self,
        _args: mp_writer_mcp_server::features::parliament::FetchLegislationArgs,
    ) -> Result<serde_json::Value, AppError> {
        self.record_call("legislation").await;
        Ok(self.legislation.clone())
    }
}

#[tokio::test]
async fn research_service_caches_results() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db = sled::open(temp_dir.path()).expect("sled open");
    let tree = db.open_tree("research").expect("tree");

    let config = Arc::new(AppConfig {
        port: 0,
        api_key: "test".to_string(),
        disable_proxy: false,
        cache_enabled: true,
        relevance_threshold: 0.5,
        cache_ttl: CacheTtlConfig {
            members: 10,
            bills: 10,
            legislation: 10,
            data: 10,
            research: 3600,
        },
        db_path: temp_dir.path().to_string_lossy().to_string(),
    });

    let mock = Arc::new(MockParliamentDataSource::new());
    let data_source: Arc<dyn ParliamentDataSource> = mock.clone();
    let service = ResearchService::new(config, data_source, tree);

    let request = ResearchRequestDto {
        topic: "Climate action".to_string(),
        bill_keywords: vec![],
        debate_keywords: vec![],
        mp_id: None,
        include_state_of_parties: true,
        limit: Some(3),
    };

    let first = service
        .run_research(request.clone())
        .await
        .expect("first call");
    assert!(!first.cached, "first call should not be cached");
    assert_eq!(first.bills.len(), 1);
    assert_eq!(first.votes.len(), 1);
    assert_eq!(first.legislation.len(), 1);
    assert_eq!(first.debates.len(), 1);
    assert!(first.state_of_parties.is_some());
    assert!(
        first.advisories.is_empty(),
        "expected no advisories for successful run"
    );

    let second = service.run_research(request).await.expect("second call");
    assert!(second.cached, "second call should read from cache");

    assert_eq!(mock.count_for("bills").await, 1);
    assert_eq!(mock.count_for("commonsdivisions").await, 1);
    assert_eq!(mock.count_for("commonsdebates").await, 1);
    assert_eq!(mock.count_for("stateofparties").await, 1);
    assert_eq!(mock.count_for("legislation").await, 1);
}
