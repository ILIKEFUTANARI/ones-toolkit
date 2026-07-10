mod support;

use http::{HeaderName, HeaderValue};
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use std::collections::HashMap;
use support::spawn_test_app;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_http_exposes_issue_tooling() {
    let app = spawn_test_app().await;
    let mut headers = HashMap::new();
    headers.insert(
        HeaderName::from_static("x-ones-email"),
        HeaderValue::from_static("alex@example.test"),
    );
    headers.insert(
        HeaderName::from_static("x-ones-password"),
        HeaderValue::from_static("dummy-password"),
    );

    let config = StreamableHttpClientTransportConfig::with_uri(format!("{}/mcp", app.app_base))
        .custom_headers(headers);
    let transport = StreamableHttpClientTransport::from_config(config);
    let client = ().serve(transport).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(
        tools
            .iter()
            .any(|tool| tool.name == "get_issue_by_display_id")
    );
    assert_eq!(tools.len(), 7);
    assert!(tools.iter().any(|tool| tool.name == "list_issue_activity"));

    let result = client
        .call_tool(
            CallToolRequestParams::new("get_issue_by_display_id").with_arguments(
                serde_json::json!({ "display_id": "DEMO-1234" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .unwrap();

    let structured = result.structured_content.expect("structured content");
    assert_eq!(structured["issue"]["summary"]["display_id"], "DEMO-1234");
    assert!(structured["issue"].get("detail_form").is_none());

    let activity = client
        .call_tool(
            CallToolRequestParams::new("list_issue_activity").with_arguments(
                serde_json::json!({ "issue_uuid": "issue-demo-001", "limit": 1 })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .unwrap();
    let structured = activity.structured_content.expect("structured content");
    assert_eq!(
        structured["activity"]["activity"].as_array().unwrap().len(),
        1
    );
}
