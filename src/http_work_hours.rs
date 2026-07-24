use crate::{
    api::{WorkHourMutation, WorkHourReport, WorkHoursResponse},
    error::AppResult,
    http_api::HttpState,
    work_hours::{
        AddWorkHourInput, DeleteWorkHourInput, IssueSelector, ListIssueWorkHoursQuery,
        UpdateRemainingHoursInput, WorkHourReportQuery,
    },
};
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ListIssueWorkHoursHttpQuery {
    issue_uuid: Option<String>,
    display_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
    owner_uuid: Option<String>,
    redact: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/work-hours/issue",
    params(
        ("issue_uuid" = Option<String>, Query),
        ("display_id" = Option<String>, Query),
        ("from" = Option<String>, Query),
        ("to" = Option<String>, Query),
        ("owner_uuid" = Option<String>, Query),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = WorkHoursResponse))
)]
pub(crate) async fn list_issue_work_hours(
    State(state): State<HttpState>,
    Query(query): Query<ListIssueWorkHoursHttpQuery>,
    headers: HeaderMap,
) -> AppResult<Json<WorkHoursResponse>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    let response = state
        .service
        .list_issue_work_hours(
            &auth,
            &ListIssueWorkHoursQuery {
                issue: IssueSelector {
                    issue_uuid: query.issue_uuid,
                    display_id: query.display_id,
                },
                from: query.from,
                to: query.to,
                owner_uuid: query.owner_uuid,
                redact: query.redact,
            },
        )
        .await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub(crate) struct WorkHourReportHttpQuery {
    issue_uuid: Option<String>,
    display_id: Option<String>,
    from: String,
    to: String,
    owner_uuid: Option<String>,
    redact: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/work-hours/report",
    params(
        ("issue_uuid" = Option<String>, Query),
        ("display_id" = Option<String>, Query),
        ("from" = String, Query),
        ("to" = String, Query),
        ("owner_uuid" = Option<String>, Query),
        ("redact" = Option<bool>, Query)
    ),
    responses((status = 200, body = WorkHourReport))
)]
pub(crate) async fn query_work_hour_report(
    State(state): State<HttpState>,
    Query(query): Query<WorkHourReportHttpQuery>,
    headers: HeaderMap,
) -> AppResult<Json<WorkHourReport>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    let response = state
        .service
        .query_work_hour_report(
            &auth,
            &WorkHourReportQuery {
                issue: (query.issue_uuid.is_some() || query.display_id.is_some()).then_some(
                    IssueSelector {
                        issue_uuid: query.issue_uuid,
                        display_id: query.display_id,
                    },
                ),
                from: query.from,
                to: query.to,
                owner_uuid: query.owner_uuid,
                redact: query.redact,
            },
        )
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/work-hours",
    request_body = AddWorkHourInput,
    responses((status = 200, body = WorkHourMutation))
)]
pub(crate) async fn add_work_hour(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(input): Json<AddWorkHourInput>,
) -> AppResult<Json<WorkHourMutation>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    Ok(Json(state.service.add_work_hour(&auth, &input).await?))
}

#[utoipa::path(
    patch,
    path = "/api/work-hours/remaining",
    request_body = UpdateRemainingHoursInput,
    responses((status = 200, body = WorkHourMutation))
)]
pub(crate) async fn update_remaining_hours(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(input): Json<UpdateRemainingHoursInput>,
) -> AppResult<Json<WorkHourMutation>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    Ok(Json(
        state.service.update_remaining_hours(&auth, &input).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/work-hours",
    request_body = DeleteWorkHourInput,
    responses((status = 200, body = WorkHourMutation))
)]
pub(crate) async fn delete_work_hour(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(input): Json<DeleteWorkHourInput>,
) -> AppResult<Json<WorkHourMutation>> {
    let auth = crate::http_api::auth_from_headers(&headers)?;
    Ok(Json(state.service.delete_work_hour(&auth, &input).await?))
}
