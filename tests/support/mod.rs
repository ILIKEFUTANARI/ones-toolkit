#![allow(dead_code)]

pub mod assertions;
pub mod mock_ones;

use axum::{Router, http::HeaderMap};
use mock_ones::{LegacyAvailability, LegacyCallCounters, MockState, build_mock_ones_router};
use ones::{AppConfig, OnesService};
#[cfg(all(feature = "http", feature = "mcp-http"))]
use ones::{McpAuthMode, build_http_app};
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::{
    net::TcpListener,
    task::JoinHandle,
    time::{Duration, sleep},
};

#[cfg(all(feature = "http", feature = "mcp-http"))]
pub struct TestApp {
    pub app_base: String,
    pub _ones_base: String,
    pub login_count: Arc<AtomicUsize>,
    pub field_list_count: Arc<AtomicUsize>,
    pub legacy_calls: Arc<LegacyCallCounters>,
    pub legacy_availability: Arc<LegacyAvailability>,
    _app_handle: JoinHandle<()>,
    _ones_handle: JoinHandle<()>,
}

#[cfg(all(feature = "http", feature = "mcp-http"))]
pub async fn spawn_test_app() -> TestApp {
    spawn_test_app_with(None).await
}

#[cfg(all(feature = "http", feature = "mcp-http"))]
pub async fn spawn_test_app_with(availability: Option<LegacyAvailability>) -> TestApp {
    let login_count = Arc::new(AtomicUsize::new(0));
    let field_list_count = Arc::new(AtomicUsize::new(0));
    let legacy_calls = Arc::new(LegacyCallCounters::default());
    let legacy_availability = Arc::new(availability.unwrap_or_default());
    let ones_state = MockState {
        login_count: login_count.clone(),
        field_list_count: field_list_count.clone(),
        legacy_calls: legacy_calls.clone(),
        legacy_availability: legacy_availability.clone(),
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
        field_list_count,
        legacy_calls,
        legacy_availability,
        _app_handle: app_handle,
        _ones_handle: ones_handle,
    }
}

pub async fn spawn_test_service() -> (
    Arc<OnesService>,
    Arc<AtomicUsize>,
    JoinHandle<()>,
    Arc<LegacyCallCounters>,
) {
    let login_count = Arc::new(AtomicUsize::new(0));
    let field_list_count = Arc::new(AtomicUsize::new(0));
    let legacy_calls = Arc::new(LegacyCallCounters::default());
    let legacy_availability = Arc::new(LegacyAvailability::default());
    let ones_state = MockState {
        login_count: login_count.clone(),
        field_list_count,
        legacy_calls: legacy_calls.clone(),
        legacy_availability,
    };
    let ones_router = build_mock_ones_router(ones_state);
    let (ones_base, ones_handle) = spawn_mock_ones(ones_router).await;
    let service = Arc::new(OnesService::new(AppConfig {
        ones_base_url: ones_base,
        team_uuid: "team-demo-001".to_string(),
    }));
    (service, login_count, ones_handle, legacy_calls)
}

pub async fn spawn_mock_ones(router: Router) -> (String, JoinHandle<()>) {
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
