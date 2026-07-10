mod support;

use reqwest::Client;
use serde_json::Value;
use std::sync::atomic::Ordering;
use support::{auth_headers, spawn_test_app, spawn_test_service};

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
    assert_eq!(views["views"][0]["uuid"], "view-demo-001", "{views}");

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
        direct_issue["issue"]["summary"]["display_id"], "DEMO-1234",
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
    assert_eq!(
        by_uuid["issue"]["summary"]["uuid"], "issue-demo-001",
        "{by_uuid}"
    );
    assert_eq!(
        by_uuid["issue"]["custom_fields"].as_array().unwrap().len(),
        3
    );
    assert_eq!(by_uuid["issue"]["custom_fields"][0]["name"], "Estimate");
    assert_eq!(by_uuid["issue"]["custom_fields"][2]["value"], "Human notes");
    assert!(by_uuid["issue"].get("detail_form").is_none());
    assert!(by_uuid["issue"].get("form_fields").is_none());
    assert!(!by_uuid.to_string().contains("list_pin_field"));

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
    assert_eq!(
        activity["activity"]["activity"].as_array().unwrap().len(),
        2
    );
    assert_eq!(activity["activity"]["activity"][0]["new_value"], "High");
    assert_eq!(activity["activity"]["page_info"]["next_cursor"], "msg2");
    assert!(!activity.to_string().contains("raw"));

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
    assert_eq!(
        next_activity["activity"]["activity"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        next_activity["activity"]["page_info"]["has_next_page"],
        false
    );

    assert_eq!(app.login_count.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_issue_lookup_serializes() {
    let (service, login_count, _ones_handle) = spawn_test_service().await;
    let issue = service
        .get_issue_by_display_id(
            &ones::models::AuthContext {
                email: "alex@example.test".to_string(),
                password: "dummy-password".to_string(),
            },
            "DEMO-1234",
        )
        .await
        .unwrap();
    let body = serde_json::to_value(&issue).unwrap();
    assert_eq!(body["summary"]["display_id"], "DEMO-1234");
    assert!(body.get("detail_item").is_none());
    assert_eq!(login_count.load(Ordering::SeqCst), 1);
}
