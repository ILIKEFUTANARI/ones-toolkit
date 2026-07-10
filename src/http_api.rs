use crate::{
    api::{
        ActivityResponse, GroupsResponse, IssueResponse, ViewResponse, ViewsResponse,
        WorkbenchIssuePage,
    },
    error::{AppError, AppResult},
    mcp::{McpAuthMode, build_mcp_service},
    models::AuthContext,
    ones::service::OnesService,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct HttpState {
    pub service: Arc<OnesService>,
}

pub fn build_http_app(
    service: Arc<OnesService>,
    mcp_auth_mode: McpAuthMode,
    mcp_path: String,
) -> Router {
    let state = HttpState {
        service: service.clone(),
    };
    let mcp_service = build_mcp_service(service, mcp_auth_mode);

    Router::new()
        .route("/api/workbench/views", get(list_views))
        .route("/api/workbench/views/{view_uuid}", get(get_view))
        .route("/api/workbench/views/{view_uuid}/groups", get(list_groups))
        .route("/api/workbench/views/{view_uuid}/issues", get(list_issues))
        .route(
            "/api/issues/by-display-id/{display_id}",
            get(get_issue_by_display_id),
        )
        .route("/api/issues/{issue_uuid}", get(get_issue))
        .route(
            "/api/issues/{issue_uuid}/activity",
            get(list_issue_activity),
        )
        .route("/openapi.json", get(openapi))
        .with_state(state)
        .nest_service(&mcp_path, mcp_service)
}

async fn list_views(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> AppResult<Json<ViewsResponse>> {
    let auth = auth_from_headers(&headers)?;
    let views = state.service.list_workbench_views(&auth).await?;
    Ok(Json(ViewsResponse { views }))
}

async fn get_view(
    State(state): State<HttpState>,
    Path(view_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<ViewResponse>> {
    let auth = auth_from_headers(&headers)?;
    let view = state.service.get_workbench_view(&auth, &view_uuid).await?;
    Ok(Json(ViewResponse { view }))
}

async fn list_groups(
    State(state): State<HttpState>,
    Path(view_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<GroupsResponse>> {
    let auth = auth_from_headers(&headers)?;
    let groups = state
        .service
        .list_workbench_issue_groups(&auth, &view_uuid)
        .await?;
    Ok(Json(GroupsResponse { groups }))
}

#[derive(Debug, Deserialize)]
struct ListIssuesQuery {
    group_value: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
    redact: Option<bool>,
}

async fn list_issues(
    State(state): State<HttpState>,
    Path(view_uuid): Path<String>,
    Query(query): Query<ListIssuesQuery>,
    headers: HeaderMap,
) -> AppResult<Json<WorkbenchIssuePage>> {
    let auth = auth_from_headers(&headers)?;
    let page = state
        .service
        .list_workbench_issues(
            &auth,
            &view_uuid,
            query.group_value.as_deref(),
            query.cursor.as_deref(),
            query.limit,
            query.redact.unwrap_or(true),
        )
        .await?;
    Ok(Json(page))
}

async fn get_issue_by_display_id(
    State(state): State<HttpState>,
    Path(display_id): Path<String>,
    Query(query): Query<RedactQuery>,
    headers: HeaderMap,
) -> AppResult<Json<IssueResponse>> {
    let auth = auth_from_headers(&headers)?;
    let issue = state
        .service
        .get_issue_by_display_id(&auth, &display_id, query.enabled())
        .await?;
    Ok(Json(IssueResponse { issue }))
}

async fn get_issue(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    Query(query): Query<RedactQuery>,
    headers: HeaderMap,
) -> AppResult<Json<IssueResponse>> {
    let auth = auth_from_headers(&headers)?;
    let issue = state
        .service
        .get_issue_detail(&auth, &issue_uuid, query.enabled())
        .await?;
    Ok(Json(IssueResponse { issue }))
}

async fn list_issue_activity(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    Query(query): Query<ActivityQuery>,
    headers: HeaderMap,
) -> AppResult<Json<ActivityResponse>> {
    let auth = auth_from_headers(&headers)?;
    let activity = state
        .service
        .list_issue_activity(
            &auth,
            &issue_uuid,
            query.cursor.as_deref(),
            query.limit,
            query.redact.unwrap_or(true),
        )
        .await?;
    Ok(Json(activity))
}

async fn openapi() -> Json<serde_json::Value> {
    Json(crate::openapi::document())
}

#[derive(Debug, Deserialize)]
struct RedactQuery {
    redact: Option<bool>,
}

impl RedactQuery {
    fn enabled(&self) -> bool {
        self.redact.unwrap_or(true)
    }
}

#[derive(Debug, Deserialize)]
struct ActivityQuery {
    cursor: Option<String>,
    limit: Option<u32>,
    redact: Option<bool>,
}

pub fn auth_from_headers(headers: &HeaderMap) -> AppResult<AuthContext> {
    let email = header_value(headers, "x-ones-email")
        .or_else(|| header_value(headers, "x-user-email"))
        .ok_or(AppError::MissingAuth)?;
    let password = header_value(headers, "x-ones-password")
        .or_else(|| header_value(headers, "x-user-password"))
        .ok_or(AppError::MissingAuth)?;
    Ok(AuthContext { email, password })
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}
