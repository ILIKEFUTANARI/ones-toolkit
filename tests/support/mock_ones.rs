use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
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
            "/project/api/ones-project/team/team-demo-001/v2/field/list",
            get(team_fields),
        )
        .route(
            "/project/api/project/team/team-demo-001/task/issue-demo-001/messages",
            get(messages),
        )
        .route(
            "/project/api/project/team/team-demo-001/items/graphql",
            post(related),
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

    let body = if query.contains("count() as total_count") {
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

async fn form_fields() -> impl IntoResponse {
    fixture(include_str!("../fixtures/form_fields.json"))
}

async fn messages() -> impl IntoResponse {
    fixture(include_str!("../fixtures/messages.json"))
}

async fn related() -> impl IntoResponse {
    fixture(include_str!("../fixtures/related.json"))
}

fn fixture(body: &'static str) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(http::header::CONTENT_TYPE, "application/json")],
        body,
    )
}
