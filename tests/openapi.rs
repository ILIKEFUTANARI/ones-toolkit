mod support;

use reqwest::Client;
use serde_json::Value;
use support::spawn_test_app;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openapi_describes_compact_redacted_responses() {
    let app = spawn_test_app().await;
    let openapi: Value = Client::new()
        .get(format!("{}/openapi.json", app.app_base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(openapi["info"]["version"], "0.2.0");
    assert!(!openapi.to_string().contains("#/$defs/"));
    assert_eq!(
        openapi["paths"]["/api/issues/{issue_uuid}"]["get"]["parameters"][1]["schema"]["default"],
        true
    );
    assert!(openapi["components"]["schemas"]["IssueDetail"].is_object());
}
