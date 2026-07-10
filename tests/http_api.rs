mod support;

use reqwest::Client;
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
        &activity,
        &next_activity,
    ] {
        assert_no_nulls(response);
    }

    assert_eq!(app.login_count.load(Ordering::SeqCst), 1);
}
