use crate::{
    error::AppResult,
    http_api::HttpState,
    workflow::{
        IssueTransitionsResponse, TransitionExecutionResponse, TransitionPreviewResponse,
        TransitionValues,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};

#[utoipa::path(
    get,
    path = "/api/issues/{issue_uuid}/transitions",
    params(("issue_uuid" = String, Path)),
    responses((status = 200, body = IssueTransitionsResponse))
)]
pub(crate) async fn list_issue_transitions(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<IssueTransitionsResponse>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    let result = state
        .service
        .list_issue_transitions(&auth, &issue_uuid)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/issues/{issue_uuid}/transition-preview",
    params(("issue_uuid" = String, Path)),
    request_body = TransitionValues,
    responses((status = 200, body = TransitionPreviewResponse))
)]
pub(crate) async fn preview_issue_transition(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    headers: HeaderMap,
    Json(values): Json<TransitionValues>,
) -> AppResult<Json<TransitionPreviewResponse>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    let result = state
        .service
        .preview_issue_transition(&auth, &issue_uuid, &values)
        .await?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/api/issues/{issue_uuid}/transition",
    params(("issue_uuid" = String, Path)),
    request_body = TransitionValues,
    responses((status = 200, body = TransitionExecutionResponse))
)]
pub(crate) async fn execute_issue_transition(
    State(state): State<HttpState>,
    Path(issue_uuid): Path<String>,
    headers: HeaderMap,
    Json(values): Json<TransitionValues>,
) -> AppResult<Json<TransitionExecutionResponse>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    let result = state
        .service
        .execute_issue_transition(&auth, &issue_uuid, &values)
        .await?;
    Ok(Json(result))
}
