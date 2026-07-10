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
use support::{
    assertions::{assert_no_nulls, keys},
    spawn_test_app,
};

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
    for name in ["get_issue_by_display_id", "get_issue_detail"] {
        let tool = tools.iter().find(|tool| tool.name == name).unwrap();
        let output_schema = serde_json::to_string(tool.output_schema.as_ref().unwrap()).unwrap();
        assert!(output_schema.contains("IssueDetail"), "{output_schema}");
        for removed in [
            "\"lookup\"",
            "\"summary\"",
            "\"core_fields\"",
            "\"field_uuid\"",
            "\"related\"",
        ] {
            assert!(!output_schema.contains(removed), "{output_schema}");
        }
    }
    for name in [
        "list_workbench_issues",
        "get_issue_by_display_id",
        "get_issue_detail",
        "list_issue_activity",
    ] {
        let tool = tools.iter().find(|tool| tool.name == name).unwrap();
        let schema = serde_json::to_value(tool).unwrap();
        assert!(schema["inputSchema"]["properties"]["redact"].is_object());
        assert!(
            !schema["inputSchema"]["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|field| field == "redact"))
        );
    }

    macro_rules! call_tool {
        ($name:literal, $arguments:expr) => {{
            client
                .call_tool(
                    CallToolRequestParams::new($name)
                        .with_arguments($arguments.as_object().unwrap().clone()),
                )
                .await
                .unwrap()
                .structured_content
                .expect("structured content")
        }};
    }

    let views = call_tool!("list_workbench_views", serde_json::json!({}));
    assert_eq!(keys(&views), ["views"].into());
    assert_eq!(keys(&views["views"][0]), ["name", "uuid"].into());

    let view = call_tool!(
        "get_workbench_view",
        serde_json::json!({ "view_uuid": "view-demo-001" })
    );
    assert_eq!(keys(&view), ["view"].into());
    assert_eq!(
        keys(&view["view"]),
        [
            "group_by",
            "hierarchy_enabled",
            "include_subtasks",
            "name",
            "uuid",
        ]
        .into()
    );

    let groups = call_tool!(
        "list_workbench_issue_groups",
        serde_json::json!({ "view_uuid": "view-demo-001" })
    );
    assert_eq!(keys(&groups), ["groups"].into());
    assert_eq!(
        keys(&groups["groups"][0]),
        ["hierarchy_count", "label", "total_count", "value"].into()
    );

    let issues = call_tool!(
        "list_workbench_issues",
        serde_json::json!({
            "view_uuid": "view-demo-001",
            "group_value": "proj-demo-001"
        })
    );
    assert_eq!(keys(&issues), ["issues", "page_info"].into());
    assert_eq!(
        keys(&issues["issues"][0]),
        [
            "assignee",
            "display_id",
            "downstream_task_count",
            "downstream_task_done_count",
            "due_date",
            "issue_type",
            "project",
            "status",
            "title",
            "uuid",
        ]
        .into()
    );
    assert_eq!(
        keys(&issues["page_info"]),
        ["has_next_page", "total_count"].into()
    );
    let assignee_alias = issues["issues"][0]["assignee"].clone();
    assert!(assignee_alias.as_str().unwrap().starts_with("用户-"));

    let structured = call_tool!(
        "get_issue_by_display_id",
        serde_json::json!({ "display_id": "DEMO-1234" })
    );
    assert_eq!(structured["issue"]["display_id"], "DEMO-1234");
    assert_eq!(structured["issue"]["assignee"], assignee_alias);
    assert!(structured["issue"].get("summary").is_none());
    assert_eq!(keys(&structured), ["issue"].into());
    let detail_keys = keys(&structured["issue"]);

    let unredacted = call_tool!(
        "get_issue_detail",
        serde_json::json!({ "issue_uuid": "issue-demo-001", "redact": false })
    );
    assert_eq!(keys(&unredacted), ["issue"].into());
    assert_eq!(keys(&unredacted["issue"]), detail_keys);
    assert_eq!(unredacted["issue"]["assignee"], "Alex Example");

    let activity = call_tool!(
        "list_issue_activity",
        serde_json::json!({ "issue_uuid": "issue-demo-001", "limit": 1 })
    );
    assert_eq!(activity["activity"].as_array().unwrap().len(), 1);
    assert_eq!(activity["activity"][0]["actor"], assignee_alias);
    assert_eq!(keys(&activity), ["activity", "page_info"].into());
    assert_eq!(
        keys(&activity["activity"][0]),
        [
            "actor",
            "field_name",
            "new_value",
            "old_value",
            "summary",
            "time",
            "type",
        ]
        .into()
    );
    assert_eq!(
        keys(&activity["page_info"]),
        ["has_next_page", "next_cursor"].into()
    );

    for response in [
        views, view, groups, issues, structured, unredacted, activity,
    ] {
        assert_no_nulls(&response);
    }
}
