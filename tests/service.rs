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
    let (service, login_count, _ones_handle, _legacy_calls) = spawn_test_service().await;
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
    let (service, _login_count, _ones_handle, _legacy_calls) = spawn_test_service().await;
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
    let (service, _login_count, _ones_handle, _legacy_calls) = spawn_test_service().await;
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
    let (service, _login_count, _ones_handle, _legacy_calls) = spawn_test_service().await;
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
    let (service, _login_count, _ones_handle, _legacy_calls) = spawn_test_service().await;
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

mod legacy_fallback {
    use super::auth;
    use crate::support::mock_ones::{
        LegacyAvailability, LegacyCallCounters, MockState, build_mock_ones_router,
    };
    use crate::support::spawn_mock_ones;
    use ones::{AppConfig, OnesService};
    use std::sync::{Arc, atomic::Ordering};

    async fn spawn(
        availability: LegacyAvailability,
    ) -> (
        OnesService,
        Arc<LegacyCallCounters>,
        tokio::task::JoinHandle<()>,
    ) {
        let login_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let legacy_calls = Arc::new(LegacyCallCounters::default());
        let legacy_availability = Arc::new(availability);
        let ones_state = MockState {
            login_count,
            field_list_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            legacy_calls: legacy_calls.clone(),
            legacy_availability,
        };
        let router = build_mock_ones_router(ones_state);
        let (ones_base, handle) = spawn_mock_ones(router).await;
        let service = OnesService::new(AppConfig {
            ones_base_url: ones_base,
            team_uuid: "team-demo-001".to_string(),
        });
        (service, legacy_calls, handle)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_prefer_public_endpoint_when_available() {
        let (service, calls, _h) = spawn(LegacyAvailability::default()).await;
        let _ = service.list_team_fields(&auth()).await.unwrap();
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_fall_back_on_public_404() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let _ = service.list_team_fields(&auth()).await.unwrap();
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn transitions_prefer_public_endpoint_when_available() {
        let (service, calls, _h) = spawn(LegacyAvailability::default()).await;
        let _ = service
            .list_issue_transitions(&auth(), "issue-demo-001")
            .await
            .unwrap();
        assert_eq!(calls.transitions_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.transitions_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn transitions_fall_back_on_public_404() {
        let availability = LegacyAvailability {
            transitions: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let _ = service
            .list_issue_transitions(&auth(), "issue-demo-001")
            .await
            .unwrap();
        assert_eq!(calls.transitions_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.transitions_internal.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_fall_back_on_public_405() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            fields_status: Arc::new(std::sync::Mutex::new(405)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let _ = service.list_team_fields(&auth()).await.unwrap();
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_do_not_fall_back_on_401() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            fields_status: Arc::new(std::sync::Mutex::new(401)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let err = service.list_team_fields(&auth()).await.unwrap_err();
        assert!(err.to_string().contains("401"));
        // 401 triggers the login-retry path so the public endpoint is hit twice.
        assert!(calls.fields_public.load(Ordering::SeqCst) >= 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_do_not_fall_back_on_403() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            fields_status: Arc::new(std::sync::Mutex::new(403)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let err = service.list_team_fields(&auth()).await.unwrap_err();
        assert!(err.to_string().contains("403"));
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_do_not_fall_back_on_400() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            fields_status: Arc::new(std::sync::Mutex::new(400)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let err = service.list_team_fields(&auth()).await.unwrap_err();
        assert!(err.to_string().contains("400"));
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_do_not_fall_back_on_429() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            fields_status: Arc::new(std::sync::Mutex::new(429)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let err = service.list_team_fields(&auth()).await.unwrap_err();
        assert!(err.to_string().contains("429"));
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fields_do_not_fall_back_on_500() {
        let availability = LegacyAvailability {
            fields: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            fields_status: Arc::new(std::sync::Mutex::new(500)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let err = service.list_team_fields(&auth()).await.unwrap_err();
        assert!(err.to_string().contains("500"));
        assert_eq!(calls.fields_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.fields_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn transitions_do_not_fall_back_on_500() {
        let availability = LegacyAvailability {
            transitions: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            transitions_status: Arc::new(std::sync::Mutex::new(500)),
            ..LegacyAvailability::default()
        };
        let (service, calls, _h) = spawn(availability).await;
        let err = service
            .list_issue_transitions(&auth(), "issue-demo-001")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("500"));
        assert_eq!(calls.transitions_public.load(Ordering::SeqCst), 1);
        assert_eq!(calls.transitions_internal.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn task_info_public_hits_legacy_endpoint() {
        let availability = LegacyAvailability::default();
        let (service, calls, _h) = spawn(availability).await;
        let result = service
            .legacy_task_info(&auth(), "issue-demo-001")
            .await
            .unwrap();
        assert_eq!(result.uuid, "issue-demo-001");
        assert_eq!(result.display_id.as_deref(), Some("DEMO-1234"));
        assert_eq!(result.number, Some(1));
        assert_eq!(calls.task_info_public.load(Ordering::SeqCst), 1);
    }
}
