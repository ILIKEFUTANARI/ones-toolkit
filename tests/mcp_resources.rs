mod support;

use http::{HeaderName, HeaderValue};
use rmcp::{
    ServiceExt,
    model::{ReadResourceRequestParams, ResourceContents},
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::Value;
use std::{collections::HashMap, sync::atomic::Ordering};
use support::{assertions::assert_no_nulls, spawn_test_app};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mcp_exposes_team_and_issue_field_resources() {
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
    let transport = StreamableHttpClientTransport::from_config(
        StreamableHttpClientTransportConfig::with_uri(format!("{}/mcp", app.app_base))
            .custom_headers(headers),
    );
    let client = ().serve(transport).await.unwrap();

    let resources = client.list_all_resources().await.unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].uri, "ones://fields");
    let templates = client.list_all_resource_templates().await.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(
        templates[0].uri_template,
        "ones://issues/{issue_uuid}/fields"
    );

    let team = read_json_resource(&client, "ones://fields").await;
    assert_eq!(team["fields"].as_array().unwrap().len(), 18);
    assert!(team["fields"].as_array().unwrap().iter().any(|field| {
        field["uuid"] == "ownerDynamic"
            && field["type_uuid"] == "assign"
            && field["can_group"] == true
    }));
    assert_no_nulls(&team);

    let second = read_json_resource(&client, "ones://fields").await;
    assert_eq!(second, team);
    assert_eq!(app.field_list_count.load(Ordering::SeqCst), 1);

    let issue = read_json_resource(&client, "ones://issues/issue-demo-001/fields").await;
    assert!(issue["fields"].as_array().unwrap().iter().any(|field| {
        field["uuid"] == "CustomUser4" && field["reference_object_type"] == "user"
    }));
    assert_no_nulls(&issue);

    assert!(
        client
            .read_resource(ReadResourceRequestParams::new("ones://unknown"))
            .await
            .is_err()
    );
}

async fn read_json_resource(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    uri: &str,
) -> Value {
    let result = client
        .read_resource(ReadResourceRequestParams::new(uri))
        .await
        .unwrap();
    let text = match &result.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => text,
        _ => panic!("expected text resource"),
    };
    serde_json::from_str(text).unwrap()
}
