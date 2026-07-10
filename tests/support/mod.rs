#![allow(dead_code)]

pub mod assertions;

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use ones::{AppConfig, McpAuthMode, OnesService, build_http_app};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::{
    net::TcpListener,
    task::JoinHandle,
    time::{Duration, sleep},
};

#[derive(Clone)]
pub struct MockState {
    pub login_count: Arc<AtomicUsize>,
}

pub struct TestApp {
    pub app_base: String,
    pub _ones_base: String,
    pub login_count: Arc<AtomicUsize>,
    _app_handle: JoinHandle<()>,
    _ones_handle: JoinHandle<()>,
}

pub async fn spawn_test_app() -> TestApp {
    let login_count = Arc::new(AtomicUsize::new(0));
    let ones_state = MockState {
        login_count: login_count.clone(),
    };

    let ones_router = build_mock_ones_router(ones_state);
    let (ones_base, ones_handle) = spawn_mock_ones(ones_router).await;

    let service = Arc::new(OnesService::new(AppConfig {
        ones_base_url: ones_base.clone(),
        team_uuid: "team-demo-001".to_string(),
    }));
    let app = build_http_app(service, McpAuthMode::HttpHeaders, "/mcp".to_string());
    let app_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let app_addr = app_listener.local_addr().unwrap();
    let app_handle = tokio::spawn(async move {
        axum::serve(app_listener, app).await.unwrap();
    });
    wait_until_ready(&format!("http://{}/openapi.json", app_addr)).await;

    TestApp {
        app_base: format!("http://{}", app_addr),
        _ones_base: ones_base,
        login_count,
        _app_handle: app_handle,
        _ones_handle: ones_handle,
    }
}

pub async fn spawn_test_service() -> (Arc<OnesService>, Arc<AtomicUsize>, JoinHandle<()>) {
    let login_count = Arc::new(AtomicUsize::new(0));
    let ones_state = MockState {
        login_count: login_count.clone(),
    };
    let ones_router = build_mock_ones_router(ones_state);
    let (ones_base, ones_handle) = spawn_mock_ones(ones_router).await;
    let service = Arc::new(OnesService::new(AppConfig {
        ones_base_url: ones_base,
        team_uuid: "team-demo-001".to_string(),
    }));
    (service, login_count, ones_handle)
}

async fn spawn_mock_ones(router: Router) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let base = format!("http://{}", addr);
    wait_until_ready(&format!(
        "{base}/project/api/project/team/team-demo-001/user_filter_views/list"
    ))
    .await;
    (base, handle)
}

async fn wait_until_ready(url: &str) {
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client.get(url).send().await.is_ok() {
            return;
        }
        sleep(Duration::from_millis(20)).await;
    }
    panic!("timed out waiting for {url}");
}

pub fn auth_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-ones-email", "alex@example.test".parse().unwrap());
    headers.insert("x-ones-password", "dummy-password".parse().unwrap());
    headers
}

fn build_mock_ones_router(state: MockState) -> Router {
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

async fn workitems_onesql(State(_): State<MockState>, request: Request<Body>) -> impl IntoResponse {
    let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let query = value["query"].as_str().unwrap_or_default();

    let body = if query.contains("count() as total_count") {
        include_str!("../fixtures/groups.json")
    } else if query.contains("where uid(uuid) = uid('issue-demo-001')") {
        assert!(!query.contains("field004"), "{query}");
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
        assert!(!query.contains("field057"), "{query}");
        assert!(!query.contains("LinkedIssue6"), "{query}");
        include_str!("../fixtures/issue_detail.json")
    } else {
        assert!(query.contains("field004.uuid,field004.name"), "{query}");
        assert!(!query.contains("field008"), "{query}");
        include_str!("../fixtures/issue_list.json")
    };

    (
        StatusCode::OK,
        [(http::header::CONTENT_TYPE, "application/json")],
        body,
    )
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
