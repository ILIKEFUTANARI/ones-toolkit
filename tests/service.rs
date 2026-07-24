mod support;

use ones::work_hours::{
    AddWorkHourInput, DeleteWorkHourInput, IssueSelector, ListIssueWorkHoursQuery,
    UpdateRemainingHoursInput, WorkHourReportQuery,
};
use std::sync::atomic::Ordering;
use support::spawn_test_service;

fn auth() -> ones::models::AuthContext {
    ones::models::AuthContext {
        email: "alex@example.test".to_string(),
        password: "dummy-password".to_string(),
    }
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
            true,
        )
        .await
        .unwrap();
    let body = serde_json::to_value(&issue).unwrap();
    assert_eq!(body["display_id"], "DEMO-1234");
    assert!(body.get("lookup").is_none());
    assert_eq!(login_count.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_lists_issue_work_hours_with_display_id_and_filters() {
    let (service, _login_count, _ones_handle) = spawn_test_service().await;
    let response = service
        .list_issue_work_hours(
            &auth(),
            &ListIssueWorkHoursQuery {
                issue: IssueSelector {
                    issue_uuid: None,
                    display_id: Some("DEMO-1234".to_string()),
                },
                from: Some("2026-07-19".to_string()),
                to: Some("2026-07-20".to_string()),
                owner_uuid: Some("session-user-demo".to_string()),
                redact: Some(false),
            },
        )
        .await
        .unwrap();

    assert_eq!(response.issue.issue_uuid, "issue-demo-001");
    assert_eq!(response.issue.display_id.as_deref(), Some("DEMO-1234"));
    assert_eq!(response.entries[0].hours, 1.25);
    assert_eq!(response.entries[0].start_at, "2026-07-20T09:00:00Z");
    assert_eq!(
        response.entries[0].owner.name.as_deref(),
        Some("Alice Example")
    );
    assert_eq!(
        response.entries[1].hour_type,
        ones::work_hours::WorkHourType::Estimated
    );
    assert_eq!(response.summaries[0].actual_hours, 1.25);
    assert_eq!(response.summaries[0].estimated_hours, 2.5);
    assert_eq!(response.summaries[0].remaining_hours, 1.25);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_reports_all_or_one_issue_and_converts_units() {
    let (service, _login_count, _ones_handle) = spawn_test_service().await;
    let all = service
        .query_work_hour_report(
            &auth(),
            &WorkHourReportQuery {
                issue: None,
                from: "2026-07-19".to_string(),
                to: "2026-07-20".to_string(),
                owner_uuid: Some("session-user-demo".to_string()),
                redact: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(all.buckets.len(), 2);
    assert_eq!(all.buckets[0].actual_hours, 3.75);
    assert_eq!(all.buckets[0].series[1].hours, 2.5);

    let one = service
        .query_work_hour_report(
            &auth(),
            &WorkHourReportQuery {
                issue: Some(IssueSelector {
                    issue_uuid: Some("issue-demo-001".to_string()),
                    display_id: None,
                }),
                from: "2026-07-19".to_string(),
                to: "2026-07-20".to_string(),
                owner_uuid: None,
                redact: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(one.buckets.len(), 1);
    assert_eq!(one.buckets[0].task.uuid, "issue-demo-001");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_adds_updates_and_deletes_work_hours() {
    let (service, _login_count, _ones_handle) = spawn_test_service().await;
    let added = service
        .add_work_hour(
            &auth(),
            &AddWorkHourInput {
                issue: IssueSelector {
                    issue_uuid: Some("issue-demo-001".to_string()),
                    display_id: None,
                },
                hours: 1.25,
                description: "Reviewed the payment flow".to_string(),
                start_at: Some("2026-07-20T09:00:00Z".to_string()),
                owner_uuid: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(added.key, "manhour-demo-added");

    let updated = service
        .update_remaining_hours(
            &auth(),
            &UpdateRemainingHoursInput {
                issue: IssueSelector {
                    issue_uuid: Some("issue-demo-001".to_string()),
                    display_id: None,
                },
                remaining_hours: 0.0,
                owner_uuid: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.remaining_hours, Some(0.0));

    for (key, expected_type) in [
        (
            "manhour-demo-recorded",
            ones::work_hours::WorkHourType::Recorded,
        ),
        (
            "manhour-demo-estimated",
            ones::work_hours::WorkHourType::Estimated,
        ),
    ] {
        let deleted = service
            .delete_work_hour(
                &auth(),
                &DeleteWorkHourInput {
                    issue: IssueSelector {
                        issue_uuid: Some("issue-demo-001".to_string()),
                        display_id: None,
                    },
                    key: key.to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(deleted.key, key);
        assert_eq!(deleted.deleted_type, Some(expected_type));
    }

    let missing = service
        .delete_work_hour(
            &auth(),
            &DeleteWorkHourInput {
                issue: IssueSelector {
                    issue_uuid: Some("issue-demo-001".to_string()),
                    display_id: None,
                },
                key: "manhour-not-on-issue".to_string(),
            },
        )
        .await
        .unwrap_err();
    assert!(missing.to_string().contains("resource not found"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_surfaces_graphql_permission_errors() {
    let (service, _login_count, _ones_handle) = spawn_test_service().await;
    let error = service
        .update_remaining_hours(
            &auth(),
            &UpdateRemainingHoursInput {
                issue: IssueSelector {
                    issue_uuid: Some("issue-denied".to_string()),
                    display_id: None,
                },
                remaining_hours: 2.0,
                owner_uuid: None,
            },
        )
        .await
        .unwrap_err();
    assert!(error.to_string().contains("work-hour permission denied"));
}
