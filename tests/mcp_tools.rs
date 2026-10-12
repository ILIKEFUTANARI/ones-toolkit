#![cfg(all(feature = "http", feature = "mcp-http"))]

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
    assert_eq!(tools.len(), 19);
    assert!(tools.iter().any(|tool| tool.name == "list_issue_activity"));
    assert!(
        tools
            .iter()
            .any(|tool| tool.name == "list_issue_attachments")
    );
    assert!(
        tools
            .iter()
            .any(|tool| tool.name == "list_issue_code_links")
    );
    assert!(tools.iter().any(|tool| tool.name == "create_issue_branch"));
    assert!(
        tools
            .iter()
            .any(|tool| tool.name == "list_issue_transitions")
    );
    assert!(
        tools
            .iter()
            .any(|tool| tool.name == "preview_issue_transition")
    );
    assert!(
        tools
            .iter()
            .any(|tool| tool.name == "execute_issue_transition")
    );
    let task_info_tool = tools
        .iter()
        .find(|tool| tool.name == "get_issue_task_info")
        .expect("task-info tool present");
    let task_info_schema = serde_json::to_value(task_info_tool).unwrap();
    let task_info_output_schema =
        serde_json::to_string(task_info_tool.output_schema.as_ref().unwrap()).unwrap();
    assert!(
        task_info_schema["inputSchema"]["properties"]["issue_uuid"].is_object(),
        "{task_info_schema}"
    );
    assert!(
        task_info_output_schema.contains("LegacyTaskInfo")
            || task_info_output_schema.contains("task_info"),
        "{task_info_output_schema}"
    );
    for name in ["preview_issue_transition", "execute_issue_transition"] {
        let tool = tools.iter().find(|tool| tool.name == name).unwrap();
        let schema = serde_json::to_value(tool).unwrap();
        let value_schema = &schema["inputSchema"]["$defs"]["TransitionFieldValueSchema"];
        assert!(value_schema["anyOf"].is_array(), "{schema}");
        let branches = value_schema["anyOf"].as_array().unwrap();
        assert_eq!(branches.len(), 3, "{schema}");
        assert!(branches.iter().any(|branch| branch["type"] == "string"));
        assert!(branches.iter().any(|branch| branch["type"] == "object"));
        assert!(
            branches
                .iter()
                .any(|branch| branch["type"] == "array" && branch["items"]["type"] == "object")
        );
    }
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

    for (name, required) in [
        ("list_issue_work_hours", "issue"),
        ("query_work_hour_report", "from"),
        ("add_work_hour", "hours"),
        ("update_remaining_hours", "remaining_hours"),
        ("delete_work_hour", "key"),
    ] {
        let tool = tools.iter().find(|tool| tool.name == name).unwrap();
        let schema = serde_json::to_value(tool).unwrap();
        assert!(schema["inputSchema"]["properties"].get(required).is_some());
        assert!(
            schema["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == required)
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

    let task_info = call_tool!(
        "get_issue_task_info",
        serde_json::json!({ "issue_uuid": "issue-demo-001" })
    );
    assert_eq!(keys(&task_info), ["task_info"].into());
    assert_eq!(task_info["task_info"]["uuid"], "issue-demo-001");
    assert_eq!(task_info["task_info"]["display_id"], "DEMO-1234");
    assert_eq!(task_info["task_info"]["number"], 1);
    assert_eq!(
        keys(&task_info["task_info"]),
        [
            "assign",
            "create_time",
            "deadline",
            "desc",
            "display_id",
            "issue_type_uuid",
            "number",
            "owner",
            "path",
            "priority_uuid",
            "project_uuid",
            "sprint_uuid",
            "status_uuid",
            "summary",
            "update_time",
            "uuid",
        ]
        .into()
    );

    let attachments = call_tool!(
        "list_issue_attachments",
        serde_json::json!({ "issue_uuid": "issue-demo-001" })
    );
    assert_eq!(keys(&attachments), ["attachments", "page_info"].into());
    assert_eq!(
        keys(&attachments["attachments"][0]),
        ["download_url", "mime", "name", "size", "uuid"].into()
    );
    assert_eq!(
        attachments["attachments"][0]["download_url"],
        "https://ones.example.test/download/design.txt?signature=test"
    );
    assert_eq!(
        keys(&attachments["page_info"]),
        ["has_next_page", "next_cursor", "total_count"].into()
    );
    assert_eq!(attachments["page_info"]["next_cursor"], "100");

    let next_attachments = call_tool!(
        "list_issue_attachments",
        serde_json::json!({ "issue_uuid": "issue-demo-001", "cursor": "100" })
    );
    assert_eq!(next_attachments["attachments"].as_array().unwrap().len(), 1);
    assert_eq!(next_attachments["attachments"][0]["name"], "notes.txt");
    assert_eq!(next_attachments["page_info"]["has_next_page"], false);

    let code_links = call_tool!(
        "list_issue_code_links",
        serde_json::json!({ "issue_uuid": "issue-demo-001" })
    );
    assert_eq!(
        keys(&code_links),
        ["branches", "commits", "pull_requests", "repositories"].into()
    );
    assert_eq!(code_links["repositories"][0]["uuid"], "repo-demo-001");
    assert_eq!(code_links["branches"][0]["name"], "feature/demo");
    assert!(
        code_links["commits"][0]["author"]
            .as_str()
            .unwrap()
            .starts_with("用户-")
    );
    assert_eq!(code_links["pull_requests"][0]["number"], 42);

    let branch = call_tool!(
        "create_issue_branch",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "repo_uuid": "repo-demo-001",
            "source_branch": "main",
            "target_branch": "feature/demo"
        })
    );
    assert_eq!(branch["created"], true);
    assert_eq!(branch["target_branch"], "feature/demo");

    let transitions = call_tool!(
        "list_issue_transitions",
        serde_json::json!({ "issue_uuid": "issue-demo-001" })
    );
    assert_eq!(transitions["transitions"].as_array().unwrap().len(), 2);
    assert_eq!(
        transitions["transitions"][0]["to_status"]["name"],
        "FAT测试"
    );
    assert_eq!(
        transitions["transitions"][0]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["uuid"] == "field061")
            .unwrap()["semantic"],
        "worklog"
    );

    let invalid_preview = call_tool!(
        "preview_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001"
        })
    );
    assert_eq!(invalid_preview["valid"], false);
    assert!(
        invalid_preview["missing_required_fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "已登记工时")
    );

    let valid_preview = call_tool!(
        "preview_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001",
            "worklog": {
                "hours": 6.0,
                "description": "完成接口改造",
                "start_at": "2026-07-14T09:00:00Z"
            },
            "test_scope": "覆盖主流程与异常分支",
            "field_values": [{
                "field_uuid": "field-demo-text",
                "value": "已完成接口回归"
            }, {
                "field_uuid": "field057",
                "value": [{
                    "uuid": "comment-demo-001",
                    "content": "FAT 回归评论"
                }]
            }]
        })
    );
    assert_eq!(valid_preview["valid"], true);
    assert_eq!(
        valid_preview["transition"]["from_status"]["name"],
        "开发/自测"
    );
    assert_eq!(valid_preview["transition"]["to_status"]["name"], "FAT测试");
    assert_eq!(valid_preview["validation_errors"], serde_json::json!([]));
    assert_eq!(
        valid_preview["normalized_field_values"]
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["field_uuid"] == "field057")
            .unwrap()["value"],
        serde_json::json!([{
            "uuid": "comment-demo-001",
            "content": "FAT 回归评论"
        }])
    );

    let object_preview = call_tool!(
        "preview_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001",
            "worklog": {
                "hours": 6.0,
                "description": "对象值兼容回归",
                "start_at": "2026-07-14T09:00:00Z"
            },
            "test_scope": "对象值兼容",
            "field_values": [{
                "field_uuid": "field057",
                "value": {
                    "uuid": "comment-demo-002",
                    "content": "单对象兼容"
                }
            }]
        })
    );
    assert_eq!(object_preview["valid"], true);

    let invalid_type_preview = call_tool!(
        "preview_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001",
            "field_values": [{
                "field_uuid": "field-demo-text",
                "value": 123
            }]
        })
    );
    assert_eq!(invalid_type_preview["valid"], false);
    assert!(
        invalid_type_preview["validation_errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error.as_str().unwrap().contains("field-demo-text"))
    );

    for invalid_value in [
        serde_json::json!(null),
        serde_json::json!(true),
        serde_json::json!([123]),
    ] {
        let invalid_reference_preview = call_tool!(
            "preview_issue_transition",
            serde_json::json!({
                "issue_uuid": "issue-demo-001",
                "transition_uuid": "transition-demo-001",
                "field_values": [{
                    "field_uuid": "field057",
                    "value": invalid_value
                }]
            })
        );
        assert_eq!(invalid_reference_preview["valid"], false);
        assert!(
            invalid_reference_preview["validation_errors"]
                .as_array()
                .unwrap()
                .iter()
                .any(|error| error.as_str().unwrap().contains("field057"))
        );
    }

    let over_limit_preview = call_tool!(
        "preview_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001",
            "worklog": {
                "hours": 11.0,
                "description": "完成接口改造",
                "start_at": "2026-07-14T09:00:00Z"
            },
            "test_scope": "覆盖主流程与异常分支"
        })
    );
    assert_eq!(over_limit_preview["valid"], false);
    assert!(
        over_limit_preview["validation_errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error.as_str().unwrap().contains("recordable limit"))
    );

    let no_field_preview = call_tool!(
        "preview_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-002"
        })
    );
    assert_eq!(no_field_preview["valid"], true);

    let transition = call_tool!(
        "execute_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001",
            "worklog": {
                "hours": 6.0,
                "description": "完成接口改造",
                "start_at": "2026-07-14T09:00:00Z"
            },
            "test_scope": "覆盖主流程与异常分支",
            "field_values": [{
                "field_uuid": "field-demo-text",
                "value": "已完成接口回归"
            }, {
                "field_uuid": "field057",
                "value": [{
                    "uuid": "comment-demo-001",
                    "content": "FAT 回归评论"
                }]
            }]
        })
    );
    assert_eq!(transition["transition_uuid"], "transition-demo-001");
    assert_eq!(transition["target_status"]["name"], "FAT测试");

    let missing_compute_transition = call_tool!(
        "execute_issue_transition",
        serde_json::json!({
            "issue_uuid": "issue-demo-001",
            "transition_uuid": "transition-demo-001",
            "worklog": {
                "hours": 6.0,
                "description": "缺少 compute token 的异步回归",
                "start_at": "2026-07-14T09:00:00Z"
            },
            "test_scope": "异步令牌兼容",
            "field_values": [{
                "field_uuid": "field-demo-text",
                "value": "simulate-missing-compute"
            }]
        })
    );
    assert_eq!(
        missing_compute_transition["transition_uuid"],
        "transition-demo-001"
    );

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

    let work_hours = call_tool!(
        "list_issue_work_hours",
        serde_json::json!({
            "issue": {"issue_uuid": "issue-demo-001"},
            "from": "2026-07-19",
            "to": "2026-07-20"
        })
    );
    assert_eq!(work_hours["entries"][0]["hours"], 1.25);
    assert_eq!(work_hours["summaries"][0]["actual_hours"], 1.25);

    let work_report = call_tool!(
        "query_work_hour_report",
        serde_json::json!({
            "from": "2026-07-19",
            "to": "2026-07-20"
        })
    );
    assert_eq!(work_report["buckets"].as_array().unwrap().len(), 2);
    assert_eq!(work_report["buckets"][0]["actual_hours"], 3.75);

    let added = call_tool!(
        "add_work_hour",
        serde_json::json!({
            "issue": {"issue_uuid": "issue-demo-001"},
            "hours": 1.25,
            "description": "Reviewed the payment flow",
            "start_at": "2026-07-20T09:00:00Z"
        })
    );
    assert_eq!(added["key"], "manhour-demo-added");

    let updated = call_tool!(
        "update_remaining_hours",
        serde_json::json!({
            "issue": {"issue_uuid": "issue-demo-001"},
            "remaining_hours": 0
        })
    );
    assert_eq!(updated["remaining_hours"], 0.0);

    let deleted = call_tool!(
        "delete_work_hour",
        serde_json::json!({
            "issue": {"display_id": "DEMO-1234"},
            "key": "manhour-demo-recorded"
        })
    );
    assert_eq!(deleted["key"], "manhour-demo-recorded");
    assert_eq!(deleted["deleted_type"], "recorded");

    for response in [
        views,
        view,
        groups,
        issues,
        structured,
        unredacted,
        task_info,
        attachments,
        next_attachments,
        code_links,
        branch,
        transitions,
        invalid_preview,
        valid_preview,
        transition,
        activity,
        work_hours,
        work_report,
        added,
        updated,
        deleted,
    ] {
        assert_no_nulls(&response);
    }
}
