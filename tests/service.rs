mod support;

use std::sync::atomic::Ordering;
use support::spawn_test_service;

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
            true,
        )
        .await
        .unwrap();
    let body = serde_json::to_value(&issue).unwrap();
    assert_eq!(body["display_id"], "DEMO-1234");
    assert!(body.get("lookup").is_none());
    assert_eq!(login_count.load(Ordering::SeqCst), 1);
}
