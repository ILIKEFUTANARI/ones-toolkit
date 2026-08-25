use crate::{
    api::{
        ActivityResponse, AttachmentsResponse, CodeLinksResponse, CreateBranchResponse,
        GroupsResponse, IssueResponse, LegacyTaskInfoResponse, ViewResponse, ViewsResponse,
        WorkbenchIssuePage,
    },
    code::CreateBranchInput,
    error::{AppError, AppResult},
    http_work_hours, http_workflow,
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
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Clone)]
pub struct HttpState {
    pub service: Arc<OnesService>,
    pub openapi: Arc<serde_json::Value>,
}

pub fn build_http_app(
    service: Arc<OnesService>,
    mcp_auth_mode: McpAuthMode,
    mcp_path: String,
) -> Router {
    let mcp_service = build_mcp_service(service.clone(), mcp_auth_mode);
    let (router, mut document) = build_api_router().split_for_parts();
    crate::openapi::finalize(&mut document);
    let state = HttpState {
        service,
        openapi: Arc::new(serde_json::to_value(document).expect("OpenAPI should serialize")),
    };

    router
        .route("/openapi.json", get(openapi))
        .with_state(state)
        .nest_service(&mcp_path, mcp_service)
}

pub(crate) fn build_api_router() -> OpenApiRouter<HttpState> {
    OpenApiRouter::new()
        .routes(routes!(list_views))
        .routes(routes!(get_view))
        .routes(routes!(list_groups))
        .routes(routes!(list_issues))
        .routes(routes!(get_issue_by_display_id))
        .routes(routes!(get_issue))
        .routes(routes!(get_issue_task_info))
        .routes(routes!(list_issue_attachments))
        .routes(routes!(list_issue_code_links))
        .routes(routes!(create_issue_branch))
        .routes(routes!(http_workflow::list_issue_transitions))
        .routes(routes!(http_workflow::preview_issue_transition))
        .routes(routes!(http_workflow::execute_issue_transition))
        .routes(routes!(list_issue_activity))
        .routes(routes!(http_work_hours::list_issue_work_hours))
        .routes(routes!(http_work_hours::query_work_hour_report))
        .routes(routes!(http_work_hours::add_work_hour))
        .routes(routes!(http_work_hours::update_remaining_hours))
        .routes(routes!(http_work_hours::delete_work_hour))
}

#[utoipa::path(
    get,
    path = "/api/workbench/views",
    responses((status = 200, body = ViewsResponse))
)]
async fn list_views(
    State(state): State<HttpState>,
    headers: HeaderMap,
) -> AppResult<Json<ViewsResponse>> {
    let auth = auth_from_headers(&headers)?;
    let views = state.service.list_workbench_views(&auth).await?;
    Ok(Json(ViewsResponse { views }))
}

#[utoipa::path(
    get,
    path = "/api/workbench/views/{view_uuid}",
    params(("view_uuid" = String, Path)),
    responses((status = 200, body = ViewResponse))
)]
async fn get_view(
    State(state): State<HttpState>,
    Path(view_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<ViewResponse>> {
    let auth = auth_from_headers(&headers)?;
    let view = state.service.get_workbench_view(&auth, &view_uuid).await?;
    Ok(Json(ViewResponse { view }))
}

#[utoipa::path(
    get,
    path = "/api/workbench/views/{view_uuid}/groups",
    params(("view_uuid" = String, Path)),
    responses((status = 200, body = GroupsResponse))
)]
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

#[utoipa::path(
    get,
    path = "/api/workbench/views/{view_uuid}/issues",
    params(
        ("view_uuid" = String, Path),
        ("group_value" = Option<String>, Query),
        ("cursor" = Option<String>, Query),
        ("limit" = Option<u32>, Query),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = WorkbenchIssuePage))
)]
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

#[utoipa::path(
    get,
    path = "/api/issues/by-display-id/{display_id}",
    params(
        ("display_id" = String, Path),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = IssueResponse))
)]
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

#[utoipa::path(
    get,
    path = "/api/issues/{issue_uuid}",
    params(
        ("issue_uuid" = String, Path),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = IssueResponse))
)]
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

#[utoipa::path(
    get,
    path = "/api/issues/{issue_uuid}/task-info",
    params(("issue_uuid" = String, Path)),
    responses((status = 200, body = LegacyTaskInfoResponse))
)]
async fn get_issue_task_info(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<LegacyTaskInfoResponse>> {
    let auth = auth_from_headers(&headers)?;
    let task_info = state.service.legacy_task_info(&auth, &issue_uuid).await?;
    Ok(Json(LegacyTaskInfoResponse { task_info }))
}

#[utoipa::path(
    get,
    path = "/api/issues/{issue_uuid}/attachments",
    params(
        ("issue_uuid" = String, Path),
        ("cursor" = Option<String>, Query)
    ),
    responses((status = 200, body = AttachmentsResponse))
)]
async fn list_issue_attachments(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    Query(query): Query<AttachmentQuery>,
    headers: HeaderMap,
) -> AppResult<Json<AttachmentsResponse>> {
    let auth = auth_from_headers(&headers)?;
    let attachments = state
        .service
        .list_issue_attachments(&auth, &issue_uuid, query.cursor.as_deref())
        .await?;
    Ok(Json(attachments))
}

#[utoipa::path(
    get,
    path = "/api/issues/{issue_uuid}/code-links",
    params(
        ("issue_uuid" = String, Path),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = CodeLinksResponse))
)]
async fn list_issue_code_links(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    Query(query): Query<RedactQuery>,
    headers: HeaderMap,
) -> AppResult<Json<CodeLinksResponse>> {
    let auth = auth_from_headers(&headers)?;
    let links = state
        .service
        .list_issue_code_links(&auth, &issue_uuid, query.enabled())
        .await?;
    Ok(Json(links))
}

#[utoipa::path(
    post,
    path = "/api/issues/{issue_uuid}/branches",
    params(("issue_uuid" = String, Path)),
    request_body = CreateBranchInput,
    responses((status = 200, body = CreateBranchResponse))
)]
async fn create_issue_branch(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    headers: HeaderMap,
    Json(input): Json<CreateBranchInput>,
) -> AppResult<Json<CreateBranchResponse>> {
    let auth = auth_from_headers(&headers)?;
    let result = state
        .service
        .create_issue_branch(&auth, &issue_uuid, &input)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/issues/{issue_uuid}/activity",
    params(
        ("issue_uuid" = String, Path),
        ("cursor" = Option<String>, Query),
        ("limit" = Option<u32>, Query),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = ActivityResponse))
)]
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

async fn openapi(State(state): State<HttpState>) -> Json<serde_json::Value> {
    Json((*state.openapi).clone())
}

#[derive(Debug, Deserialize)]
struct RedactQuery {
    redact: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AttachmentQuery {
    cursor: Option<String>,
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
