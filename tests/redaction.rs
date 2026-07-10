mod support;

use reqwest::Client;
use serde_json::Value;
use support::{auth_headers, spawn_test_app};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_redact_false_preserves_available_upstream_names() {
    let app = spawn_test_app().await;
    let client = Client::builder().pool_max_idle_per_host(0).build().unwrap();

    let issues = get_json(
        &client,
        format!(
            "{}/api/workbench/views/view-demo-001/issues?group_value=proj-demo-001&redact=false",
            app.app_base
        ),
    )
    .await;
    assert_eq!(issues["issues"][0]["assignee"], "Alex Example");

    let detail = get_json(
        &client,
        format!("{}/api/issues/issue-demo-001?redact=false", app.app_base),
    )
    .await;
    assert_eq!(detail["issue"]["assignee"], "Alex Example", "{detail}");
    assert_eq!(
        detail["issue"]["description"],
        "Alex Example reviewed with Taylor Example and Jordan Unknown"
    );
    assert_eq!(detail["issue"]["watchers"][0], "[[RDX:already-redacted]]");

    let activity = get_json(
        &client,
        format!(
            "{}/api/issues/issue-demo-001/activity?limit=1&redact=false",
            app.app_base
        ),
    )
    .await;
    assert_eq!(activity["activity"][0]["actor"], "Alex Example");
}

async fn get_json(client: &Client, url: String) -> Value {
    client
        .get(url)
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
