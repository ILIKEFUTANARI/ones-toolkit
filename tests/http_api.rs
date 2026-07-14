mod support;

use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::atomic::Ordering;
use support::{
    assertions::{assert_no_nulls, keys},
    auth_headers, spawn_test_app,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_api_supports_issue_lookup_and_workbench_browse() {
    let app = spawn_test_app().await;
    let client = Client::builder().pool_max_idle_per_host(0).build().unwrap();

    let views: Value = client
        .get(format!("{}/api/workbench/views", app.app_base))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(keys(&views), BTreeSet::from(["views"]));
    assert_eq!(views["views"][0]["uuid"], "view-demo-001", "{views}");
    assert_eq!(keys(&views["views"][0]), BTreeSet::from(["name", "uuid"]));
    assert_no_nulls(&views);

    let view: Value = client
        .get(format!(
            "{}/api/workbench/views/view-demo-001",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(keys(&view), BTreeSet::from(["view"]));
    assert_eq!(
        keys(&view["view"]),
        BTreeSet::from([
            "group_by",
            "hierarchy_enabled",
            "include_subtasks",
            "name",
            "uuid",
        ])
    );

    let groups: Value = client
        .get(format!(
            "{}/api/workbench/views/view-demo-001/groups",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(groups["groups"][0]["value"], "proj-demo-001", "{groups}");
    assert_eq!(keys(&groups), BTreeSet::from(["groups"]));
    assert_eq!(
        keys(&groups["groups"][0]),
        BTreeSet::from(["hierarchy_count", "label", "total_count", "value"])
    );

    let issues: Value = client
        .get(format!(
            "{}/api/workbench/views/view-demo-001/issues?group_value=proj-demo-001",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(issues["issues"][0]["display_id"], "DEMO-1234", "{issues}");
    let assignee_alias = issues["issues"][0]["assignee"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(assignee_alias.starts_with("用户-"), "{issues}");
    assert!(!issues.to_string().contains("Wrong Watcher"));
    assert_eq!(keys(&issues), BTreeSet::from(["issues", "page_info"]));
    assert_eq!(
        keys(&issues["issues"][0]),
        BTreeSet::from([
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
        ])
    );
    assert_eq!(
        keys(&issues["page_info"]),
        BTreeSet::from(["has_next_page", "total_count"])
    );

    let direct_issue: Value = client
        .get(format!(
            "{}/api/issues/by-display-id/DEMO-1234",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        direct_issue["issue"]["display_id"], "DEMO-1234",
        "{direct_issue}"
    );

    let by_uuid: Value = client
        .get(format!("{}/api/issues/issue-demo-001", app.app_base))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(by_uuid["issue"]["uuid"], "issue-demo-001", "{by_uuid}");
    assert_eq!(
        by_uuid["issue"]["custom_fields"].as_array().unwrap().len(),
        4
    );
    assert_eq!(by_uuid["issue"]["custom_fields"][0]["name"], "Estimate");
    assert_eq!(by_uuid["issue"]["custom_fields"][2]["value"], "Human notes");
    assert_eq!(by_uuid["issue"]["assignee"], assignee_alias);
    assert_eq!(by_uuid["issue"]["watchers"][0], assignee_alias);
    assert_eq!(
        by_uuid["issue"]["description"],
        format!("{assignee_alias} reviewed with 用户-003 and Jordan Unknown")
    );
    assert!(!by_uuid.to_string().contains("field_uuid"));
    assert!(!by_uuid.to_string().contains("user-demo"));
    assert!(!by_uuid.to_string().contains("[[RDX:"));
    assert_ne!(by_uuid["issue"]["creator"], by_uuid["issue"]["assignee"]);
    assert_ne!(
        by_uuid["issue"]["custom_fields"][3]["value"],
        by_uuid["issue"]["assignee"]
    );
    assert_eq!(
        keys(&by_uuid["issue"]),
        BTreeSet::from([
            "assignee",
            "created_at",
            "creator",
            "custom_fields",
            "description",
            "display_id",
            "due_date",
            "issue_type",
            "modules",
            "priority",
            "project",
            "sprint",
            "status",
            "title",
            "updated_at",
            "uuid",
            "watchers",
        ])
    );
    assert_eq!(keys(&direct_issue), BTreeSet::from(["issue"]));
    assert_eq!(keys(&direct_issue["issue"]), keys(&by_uuid["issue"]));

    let attachments: Value = client
        .get(format!(
            "{}/api/issues/issue-demo-001/attachments",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        keys(&attachments),
        BTreeSet::from(["attachments", "page_info"])
    );
    assert_eq!(attachments["attachments"].as_array().unwrap().len(), 1);
    assert_eq!(
        attachments["attachments"][0]["download_url"],
        "https://ones.example.test/download/design.txt?signature=test"
    );
    assert_eq!(
        keys(&attachments["attachments"][0]),
        BTreeSet::from(["download_url", "mime", "name", "size", "uuid"])
    );
    assert_eq!(attachments["page_info"]["has_next_page"], true);
    assert_eq!(attachments["page_info"]["next_cursor"], "100");

    let next_attachments: Value = client
        .get(format!(
            "{}/api/issues/issue-demo-001/attachments?cursor=100",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(next_attachments["attachments"].as_array().unwrap().len(), 1);
    assert_eq!(next_attachments["attachments"][0]["name"], "notes.txt");
    assert_eq!(next_attachments["page_info"]["has_next_page"], false);

    let invalid_attachment_cursor = client
        .get(format!(
            "{}/api/issues/issue-demo-001/attachments?cursor=invalid",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap();
    assert_eq!(invalid_attachment_cursor.status(), StatusCode::BAD_REQUEST);

    let code_links: Value = client
        .get(format!(
            "{}/api/issues/issue-demo-001/code-links",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        keys(&code_links),
        BTreeSet::from(["branches", "commits", "pull_requests", "repositories"])
    );
    assert_eq!(code_links["repositories"][0]["uuid"], "repo-demo-001");
    assert_eq!(code_links["branches"][0]["name"], "feature/demo");
    assert!(
        code_links["commits"][0]["author"]
            .as_str()
            .unwrap()
            .starts_with("用户-")
    );

    let branch: Value = client
        .post(format!(
            "{}/api/issues/issue-demo-001/branches",
            app.app_base
        ))
        .headers(auth_headers())
        .json(&serde_json::json!({
            "repo_uuid": "repo-demo-001",
            "source_branch": "main",
            "target_branch": "feature/demo"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(branch["created"], true);
    assert_eq!(branch["issue_uuid"], "issue-demo-001");

    let transitions: Value = client
        .get(format!(
            "{}/api/issues/issue-demo-001/transitions",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(transitions["transitions"].as_array().unwrap().len(), 2);
    assert_eq!(
        transitions["transitions"][0]["to_status"]["name"],
        "FAT测试"
    );

    let openapi: Value = client
        .get(format!("{}/openapi.json", app.app_base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        openapi["paths"]["/api/issues/{issue_uuid}/transition-preview"]["post"]["requestBody"]["content"]
            ["application/json"]["schema"]["$ref"]
            == "#/components/schemas/TransitionValues"
    );
    assert!(openapi["components"]["schemas"]["TransitionValues"].is_object());
    assert_eq!(
        openapi["paths"]["/api/issues/{issue_uuid}/branches"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/CreateBranchInput"
    );

    let invalid_preview: Value = client
        .post(format!(
            "{}/api/issues/issue-demo-001/transition-preview",
            app.app_base
        ))
        .headers(auth_headers())
        .json(&serde_json::json!({
            "transition_uuid": "transition-demo-001"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(invalid_preview["valid"], false);

    let valid_body = serde_json::json!({
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
        }]
    });
    let valid_preview: Value = client
        .post(format!(
            "{}/api/issues/issue-demo-001/transition-preview",
            app.app_base
        ))
        .headers(auth_headers())
        .json(&valid_body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(valid_preview["valid"], true);
    assert_eq!(
        valid_preview["transition"]["from_status"]["name"],
        "开发/自测"
    );
    assert_eq!(valid_preview["transition"]["to_status"]["name"], "FAT测试");

    let transition: Value = client
        .post(format!(
            "{}/api/issues/issue-demo-001/transition",
            app.app_base
        ))
        .headers(auth_headers())
        .json(&valid_body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(transition["transition_uuid"], "transition-demo-001");
    assert_eq!(transition["target_status"]["name"], "FAT测试");

    let activity: Value = client
        .get(format!(
            "{}/api/issues/issue-demo-001/activity?limit=2",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(activity["activity"].as_array().unwrap().len(), 2);
    assert_eq!(activity["activity"][0]["new_value"], "High");
    assert_eq!(activity["activity"][0]["actor"], assignee_alias);
    assert_eq!(activity["page_info"]["next_cursor"], "msg2");
    assert!(!activity.to_string().contains("raw"));
    assert_eq!(keys(&activity), BTreeSet::from(["activity", "page_info"]));
    assert_eq!(
        keys(&activity["activity"][0]),
        BTreeSet::from([
            "actor",
            "field_name",
            "new_value",
            "old_value",
            "summary",
            "time",
            "type",
        ])
    );
    assert_eq!(
        activity["activity"][1]["text"],
        format!("Discussed with {assignee_alias}")
    );
    assert_eq!(
        keys(&activity["page_info"]),
        BTreeSet::from(["has_next_page", "next_cursor"])
    );

    let next_activity: Value = client
        .get(format!(
            "{}/api/issues/issue-demo-001/activity?cursor=msg2&limit=2",
            app.app_base
        ))
        .headers(auth_headers())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(next_activity["activity"].as_array().unwrap().len(), 1);
    assert_eq!(next_activity["page_info"]["has_next_page"], false);

    for response in [
        &view,
        &groups,
        &issues,
        &direct_issue,
        &by_uuid,
        &attachments,
        &next_attachments,
        &code_links,
        &branch,
        &transitions,
        &invalid_preview,
        &valid_preview,
        &transition,
        &activity,
        &next_activity,
    ] {
        assert_no_nulls(response);
    }

    assert_eq!(app.login_count.load(Ordering::SeqCst), 1);
}
