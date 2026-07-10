use crate::{
    error::{AppError, AppResult},
    mcp::{McpAuthMode, build_mcp_service},
    models::AuthContext,
    ones::service::{OnesService, WorkbenchIssuePage},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;
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
) -> AppResult<Json<serde_json::Value>> {
    let auth = auth_from_headers(&headers)?;
    let views = state.service.list_workbench_views(&auth).await?;
    Ok(Json(json!({ "views": views })))
}

async fn get_view(
    State(state): State<HttpState>,
    Path(view_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let auth = auth_from_headers(&headers)?;
    let view = state.service.get_workbench_view(&auth, &view_uuid).await?;
    Ok(Json(json!({ "view": view })))
}

async fn list_groups(
    State(state): State<HttpState>,
    Path(view_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let auth = auth_from_headers(&headers)?;
    let groups = state
        .service
        .list_workbench_issue_groups(&auth, &view_uuid)
        .await?;
    Ok(Json(json!({ "groups": groups })))
}

#[derive(Debug, Deserialize)]
struct ListIssuesQuery {
    group_value: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
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
        )
        .await?;
    Ok(Json(page))
}

async fn get_issue_by_display_id(
    State(state): State<HttpState>,
    Path(display_id): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let auth = auth_from_headers(&headers)?;
    let issue = state
        .service
        .get_issue_by_display_id(&auth, &display_id)
        .await?;
    Ok(Json(json!({ "issue": issue })))
}

async fn get_issue(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let auth = auth_from_headers(&headers)?;
    let issue = state.service.get_issue_detail(&auth, &issue_uuid).await?;
    Ok(Json(json!({ "issue": issue })))
}

async fn list_issue_activity(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    Query(query): Query<ListIssuesQuery>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let auth = auth_from_headers(&headers)?;
    let activity = state
        .service
        .list_issue_activity(&auth, &issue_uuid, query.cursor.as_deref(), query.limit)
        .await?;
    Ok(Json(json!({ "activity": activity })))
}

async fn openapi() -> Json<serde_json::Value> {
    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "ONES Issue Wrapper",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "paths": {
            "/api/workbench/views": { "get": {} },
            "/api/workbench/views/{view_uuid}": { "get": {} },
            "/api/workbench/views/{view_uuid}/groups": { "get": {} },
            "/api/workbench/views/{view_uuid}/issues": { "get": {} },
            "/api/issues/by-display-id/{display_id}": { "get": {} },
            "/api/issues/{issue_uuid}": { "get": {} },
            "/api/issues/{issue_uuid}/activity": { "get": {} }
        },
        "components": {
            "securitySchemes": {
                "onesEmailPassword": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "x-ones-email"
                }
            }
        }
    }))
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
