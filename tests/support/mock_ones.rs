use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
pub struct MockState {
    pub login_count: Arc<AtomicUsize>,
    pub field_list_count: Arc<AtomicUsize>,
}

pub fn build_mock_ones_router(state: MockState) -> Router {
    Router::new()
        .route("/project/api/project/auth/login", post(login))
        .route(
            "/project/api/project/team/team-demo-001/user_filter_views/list",
            get(view_list),
        )
        .route(
            "/project/api/project/team/team-demo-001/user_filter_views/detail",
            post(view_detail),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/tasks/identifier",
            post(identifier),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/issue_form/fields",
            post(form_fields),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/issue/issue-demo-001/v2/transitions",
            get(transitions),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/transition/{transition_uuid}",
            get(transition_detail),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/issue_form/execute_form_rule",
            post(execute_form_rule),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/tasks/check_update_permission",
            post(check_update_permission),
        )
        .route(
            "/project/api/project/team/team-demo-001/project/project-demo-001/recordable_manhour",
            post(recordable_manhour),
        )
        .route(
            "/project/api/project/team/team-demo-001/task/issue-demo-001/new_transit",
            post(new_transit),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/async_status",
            post(async_status),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/v2/field/list",
            get(team_fields),
        )
        .route(
            "/project/api/project/team/team-demo-001/task/issue-demo-001/messages",
            get(messages),
        )
        .route(
            "/project/api/project/team/team-demo-001/task/issue-demo-001/attachments",
            get(attachments),
        )
        .route(
            "/project/api/project/team/team-demo-001/res/attachment/{attachment_uuid}",
            get(attachment_download),
        )
        .route(
            "/project/api/project/team/team-demo-001/items/graphql",
            post(related),
        )
        .route(
            "/project/api/project/team/team-demo-001/devops/scm/repo/repo-demo-001/branch/add",
            post(add_branch),
        )
        .route(
            "/project/api/ones-project/team/team-demo-001/workitems/onesql",
            post(workitems_onesql),
        )
        .with_state(state)
}

async fn login(State(state): State<MockState>) -> impl IntoResponse {
    state.login_count.fetch_add(1, Ordering::SeqCst);
    (
        StatusCode::OK,
        Json(json!({
            "user": {
                "uuid": "session-user-demo",
                "token": "mock-token-123"
            }
        })),
    )
}

async fn team_fields(State(state): State<MockState>) -> impl IntoResponse {
    state.field_list_count.fetch_add(1, Ordering::SeqCst);
    fixture(include_str!("../fixtures/team_fields.json"))
}

async fn workitems_onesql(State(_): State<MockState>, request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let query = value["query"].as_str().unwrap_or_default();

    let body = if query.contains("projectDynamic.uuid,issueTypeDynamic.uuid")
        || query.contains("projectDynamic,issueTypeDynamic")
        || query.contains("field061")
    {
        include_str!("../fixtures/workflow_issue_values.json")
    } else if query.contains("count() as total_count") {
        include_str!("../fixtures/groups.json")
    } else if query.contains("where uid(uuid) = uid('issue-demo-001')") {
        assert!(
            query.contains("ownerDynamic.uuid,ownerDynamic.name"),
            "{query}"
        );
        assert!(
            query.contains("followersDynamic.uuid,followersDynamic.name"),
            "{query}"
        );
        assert!(query.contains("CustomA1.uuid,CustomA1.name"), "{query}");
        assert!(query.contains("CustomB2"), "{query}");
        assert!(query.contains("CustomRich3"), "{query}");
        assert!(!query.contains("field004"), "{query}");
        assert!(!query.contains("field057"), "{query}");
        assert!(!query.contains("LinkedIssue6"), "{query}");
        include_str!("../fixtures/issue_detail.json")
    } else {
        assert!(
            query.contains("ownerDynamic.uuid,ownerDynamic.name"),
            "{query}"
        );
        assert!(
            query.contains("statusDynamic desc, createdDynamic desc"),
            "{query}"
        );
        assert!(!query.contains("field004"), "{query}");
        assert!(!query.contains("field008"), "{query}");
        include_str!("../fixtures/issue_list.json")
    };

    fixture(body)
}

async fn view_list() -> impl IntoResponse {
    fixture(include_str!("../fixtures/view_list.json"))
}

async fn view_detail() -> impl IntoResponse {
    fixture(include_str!("../fixtures/view_detail.json"))
}

async fn identifier() -> impl IntoResponse {
    fixture(include_str!("../fixtures/identifier.json"))
}

async fn form_fields(request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    if value.get("transition_context").is_some() {
        if value["transition_context"]["transition_uuid"] == "transition-demo-002" {
            return fixture(r#"{"fields":[]}"#);
        }
        fixture(include_str!("../fixtures/transition_fields.json"))
    } else {
        fixture(include_str!("../fixtures/form_fields.json"))
    }
}

async fn transitions() -> impl IntoResponse {
    fixture(include_str!("../fixtures/transitions.json"))
}

async fn transition_detail(Path(transition_uuid): Path<String>) -> impl IntoResponse {
    if transition_uuid == "transition-demo-002" {
        fixture(include_str!("../fixtures/transition_detail_review.json"))
    } else {
        fixture(include_str!("../fixtures/transition_detail.json"))
    }
}

async fn execute_form_rule() -> impl IntoResponse {
    fixture(include_str!("../fixtures/transition_rule.json"))
}

async fn check_update_permission() -> impl IntoResponse {
    fixture(include_str!("../fixtures/permission.json"))
}

async fn recordable_manhour() -> impl IntoResponse {
    fixture(include_str!("../fixtures/recordable_manhour.json"))
}

async fn new_transit(request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["transition_uuid"], "transition-demo-001");
    assert!(
        value["field_values"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["field_uuid"] == "XgVo3CWJ")
    );
    let worklog = value["field_values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|field| field["field_uuid"] == "field061")
        .unwrap();
    assert_eq!(worklog["value"][0]["hours"], 600000);
    assert_eq!(worklog["value"][0]["owner"], "session-user-demo");
    assert_eq!(worklog["value"][0]["task"], "issue-demo-001");
    if let Some(comment) = value["field_values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|field| field["field_uuid"] == "field057")
    {
        assert_eq!(
            comment["value"],
            json!([{"uuid": "comment-demo-001", "content": "FAT 回归评论"}])
        );
    }
    if value["field_values"]
        .as_array()
        .unwrap()
        .iter()
        .any(|field| {
            field["field_uuid"] == "field-demo-text" && field["value"] == "simulate-missing-compute"
        })
    {
        fixture(include_str!("../fixtures/new_transit_missing_compute.json"))
    } else {
        fixture(include_str!("../fixtures/new_transit.json"))
    }
}

async fn async_status(request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let tokens = value["tokens"].as_object().unwrap();
    assert_eq!(tokens["async_token"], "async-demo-token");
    if let Some(async_compute_token) = tokens.get("async_compute_token") {
        assert_eq!(async_compute_token, "async-demo-compute");
    }
    fixture(include_str!("../fixtures/async_status_done.json"))
}

async fn messages() -> impl IntoResponse {
    fixture(include_str!("../fixtures/messages.json"))
}

#[derive(Deserialize)]
struct AttachmentQuery {
    since: u64,
}

async fn attachments(Query(query): Query<AttachmentQuery>) -> impl IntoResponse {
    if query.since == 100 {
        fixture(include_str!("../fixtures/attachments_next.json"))
    } else {
        assert_eq!(query.since, 0);
        fixture(include_str!("../fixtures/attachments.json"))
    }
}

async fn attachment_download(Path(attachment_uuid): Path<String>) -> impl IntoResponse {
    if attachment_uuid == "attachment-demo-002" {
        fixture(include_str!("../fixtures/attachment_download_next.json"))
    } else {
        fixture(include_str!("../fixtures/attachment_download.json"))
    }
}

async fn related(request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let query = value["query"].as_str().unwrap_or_default();
    if query.contains("FetchUserManhoursInfo") {
        assert_eq!(
            value["variables"]["manhourFilter"]["task"]["uuid_equal"],
            "issue-demo-001"
        );
        if value["variables"]["manhourFilter"]
            .get("owner_equal")
            .is_some()
        {
            assert_eq!(
                value["variables"]["manhourFilter"]["owner_equal"],
                "session-user-demo"
            );
        }
        if value["variables"]["manhourFilter"]
            .get("startTime_range")
            .is_some()
        {
            assert_eq!(
                value["variables"]["manhourFilter"]["startTime_range"]["from"],
                "2026-07-19"
            );
            assert_eq!(
                value["variables"]["manhourFilter"]["startTime_range"]["to"],
                "2026-07-20"
            );
        }
        fixture(include_str!("../fixtures/work_hours_summary.json"))
    } else if query.contains("FetchUserManhours") {
        assert_eq!(
            value["variables"]["manhourFilter"]["task"]["uuid_in"][0],
            "issue-demo-001"
        );
        if value["variables"]["manhourFilter"]
            .get("owner_in")
            .is_some()
        {
            assert_eq!(
                value["variables"]["manhourFilter"]["owner_in"][0],
                "session-user-demo"
            );
        }
        if value["variables"]["manhourFilter"]
            .get("startTime_range")
            .is_some()
        {
            assert_eq!(
                value["variables"]["manhourFilter"]["startTime_range"]["from"],
                "2026-07-19"
            );
            assert_eq!(
                value["variables"]["manhourFilter"]["startTime_range"]["to"],
                "2026-07-20"
            );
        }
        fixture(include_str!("../fixtures/work_hours_entries.json"))
    } else if query.contains("QUERY_MANHOURS") {
        let filter = &value["variables"]["filter"]["manhours"];
        if filter.get("owner_in").is_some() {
            assert_eq!(filter["owner_in"][0], "session-user-demo");
        }
        if filter.get("task").is_some() {
            assert_eq!(filter["task"]["uuid_in"][0], "issue-demo-001");
        }
        fixture(include_str!("../fixtures/work_hours_report.json"))
    } else if query.contains("mutation AddManhour") {
        let variables = &value["variables"];
        assert_eq!(variables["task"], "issue-demo-001");
        assert_eq!(variables["owner"], "session-user-demo");
        assert_eq!(variables["hours"], 125000);
        assert_eq!(variables["start_time"], 1784538000);
        fixture(include_str!("../fixtures/work_hours_add.json"))
    } else if query.contains("mutation UpdateRemainingHour") {
        let variables = &value["variables"];
        assert_eq!(
            variables["key"],
            "remaining_hour-issue-demo-001-session-user-demo"
        );
        assert_eq!(variables["hours"], 0);
        fixture(include_str!("../fixtures/work_hours_update.json"))
    } else if query.contains("mutation DeleteManhour") {
        match value["variables"]["key"].as_str() {
            Some("manhour-demo-recorded") => {
                fixture(include_str!("../fixtures/work_hours_delete.json"))
            }
            Some("manhour-demo-estimated") => {
                fixture(include_str!("../fixtures/work_hours_delete_estimated.json"))
            }
            key => panic!("unexpected work-hour delete key {key:?}"),
        }
    } else if query.contains("hasUpdateOwnAssessManHourPermission") {
        if value["variables"]["key"] == "task-issue-denied" {
            return fixture(r#"{"errors":[{"message":"work-hour permission denied"}]}"#);
        }
        assert!(query.contains("attachPermission"), "{query}");
        assert!(query.contains("manage_task_own_assess_manhour"), "{query}");
        assert!(query.contains("manage_task_assess_manhour"), "{query}");
        assert!(query.contains("manage_task_own_record_manhours"), "{query}");
        assert!(query.contains("manage_task_record_manhours"), "{query}");
        fixture(include_str!("../fixtures/work_hour_permission.json"))
    } else if query.contains("tStubTask") {
        fixture(include_str!("../fixtures/code_repositories.json"))
    } else if query.contains("devopsBranches") {
        fixture(include_str!("../fixtures/code_links.json"))
    } else {
        fixture(include_str!("../fixtures/related.json"))
    }
}

async fn add_branch(request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["task_uuid"], "issue-demo-001");
    assert_eq!(value["source_branch"], "main");
    assert_eq!(value["target_branch"], "feature/demo");
    fixture(include_str!("../fixtures/branch_add.json"))
}

fn fixture(body: &'static str) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(http::header::CONTENT_TYPE, "application/json")],
        body,
    )
}
