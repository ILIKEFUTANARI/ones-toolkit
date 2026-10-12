use crate::{
    api::{
        ActivityResponse, AttachmentsResponse, CodeLinksResponse, CreateBranchResponse,
        GroupsResponse, IssueResponse, LegacyTaskInfoResponse, ViewResponse, ViewsResponse,
        WorkbenchIssuePage,
    },
    auth::auth_from_headers,
    code::CreateBranchInput,
    error::{AppError, AppResult},
    models::AuthContext,
    ones::service::OnesService,
    work_hours::{
        AddWorkHourInput, DeleteWorkHourInput, IssueWorkHoursResponse, ListIssueWorkHoursQuery,
        UpdateRemainingHoursInput, WorkHourMutationResponse, WorkHourReportQuery,
        WorkHourReportResponse,
    },
    workflow::{
        IssueTransitionsResponse, TransitionExecutionResponse, TransitionPreviewResponse,
        TransitionValues,
    },
};
use http::request::Parts;
use rmcp::{
    Json,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    service::RequestContext,
    tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

mod resources;

#[derive(Debug, Clone)]
pub enum McpAuthMode {
    HttpHeaders,
    Static(AuthContext),
}

#[derive(Debug, Clone)]
pub struct OnesMcpServer {
    service: Arc<OnesService>,
    auth_mode: McpAuthMode,
    tool_router: ToolRouter<Self>,
}

impl OnesMcpServer {
    pub fn new(service: Arc<OnesService>, auth_mode: McpAuthMode) -> Self {
        Self {
            service,
            auth_mode,
            tool_router: Self::tool_router(),
        }
    }

    fn auth_from_context(
        &self,
        context: &RequestContext<rmcp::RoleServer>,
    ) -> AppResult<AuthContext> {
        let parts = context.extensions.get::<Parts>();
        match (&self.auth_mode, parts) {
            (McpAuthMode::Static(auth), _) => Ok(auth.clone()),
            (McpAuthMode::HttpHeaders, Some(parts)) => auth_from_headers(&parts.headers),
            (McpAuthMode::HttpHeaders, None) => Err(AppError::MissingAuth),
        }
    }
}

/// Build the MCP Streamable HTTP service. Requires the `mcp-http` feature;
/// the `mcp` feature alone still serves the stdio transport.
#[cfg(feature = "mcp-http")]
pub fn build_mcp_service(
    service: Arc<OnesService>,
    auth_mode: McpAuthMode,
) -> rmcp::transport::streamable_http_server::StreamableHttpService<
    OnesMcpServer,
    rmcp::transport::streamable_http_server::session::local::LocalSessionManager,
> {
    rmcp::transport::streamable_http_server::StreamableHttpService::new(
        move || Ok(OnesMcpServer::new(service.clone(), auth_mode.clone())),
        Default::default(),
        Default::default(),
    )
}

#[tool_router(router = tool_router)]
impl OnesMcpServer {
    #[rmcp::tool(description = "List workbench views available to the current ONES user")]
    async fn list_workbench_views(
        &self,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ViewsResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let views = map_error(self.service.list_workbench_views(&auth).await)?;
        Ok(Json(ViewsResponse { views }))
    }

    #[rmcp::tool(description = "Get one workbench view by UUID")]
    async fn get_workbench_view(
        &self,
        Parameters(WorkbencViewRequest { view_uuid }): Parameters<WorkbencViewRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ViewResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let view = map_error(self.service.get_workbench_view(&auth, &view_uuid).await)?;
        Ok(Json(ViewResponse { view }))
    }

    #[rmcp::tool(description = "List issue groups for a workbench view")]
    async fn list_workbench_issue_groups(
        &self,
        Parameters(WorkbencViewRequest { view_uuid }): Parameters<WorkbencViewRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<GroupsResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let groups = map_error(
            self.service
                .list_workbench_issue_groups(&auth, &view_uuid)
                .await,
        )?;
        Ok(Json(GroupsResponse { groups }))
    }

    #[rmcp::tool(description = "List issues in a workbench view, optionally scoped to one group")]
    async fn list_workbench_issues(
        &self,
        Parameters(request): Parameters<ListWorkbenchIssuesRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<WorkbenchIssuePage>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let page = map_error(
            self.service
                .list_workbench_issues(
                    &auth,
                    &request.view_uuid,
                    request.group_value.as_deref(),
                    request.cursor.as_deref(),
                    request.limit,
                    request.redact.unwrap_or(true),
                )
                .await,
        )?;
        Ok(Json(WorkbenchIssuePage {
            issues: page.issues,
            page_info: page.page_info,
        }))
    }

    #[rmcp::tool(
        description = "Resolve an ONES display ID like DEMO-1234 and return the issue detail"
    )]
    async fn get_issue_by_display_id(
        &self,
        Parameters(DisplayIdRequest { display_id, redact }): Parameters<DisplayIdRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<IssueResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let issue = map_error(
            self.service
                .get_issue_by_display_id(&auth, &display_id, redact.unwrap_or(true))
                .await,
        )?;
        Ok(Json(IssueResponse { issue }))
    }

    #[rmcp::tool(description = "Return the aggregated issue detail for one ONES issue UUID")]
    async fn get_issue_detail(
        &self,
        Parameters(IssueUuidRequest { issue_uuid, redact }): Parameters<IssueUuidRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<IssueResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let issue = map_error(
            self.service
                .get_issue_detail(&auth, &issue_uuid, redact.unwrap_or(true))
                .await,
        )?;
        Ok(Json(IssueResponse { issue }))
    }

    #[rmcp::tool(
        description = "Read the public legacy ONES task info record (UUID or display number) for one issue"
    )]
    async fn get_issue_task_info(
        &self,
        Parameters(IssueUuidRequest { issue_uuid, .. }): Parameters<IssueUuidRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<LegacyTaskInfoResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let task_info = map_error(self.service.legacy_task_info(&auth, &issue_uuid).await)?;
        Ok(Json(LegacyTaskInfoResponse { task_info }))
    }

    #[rmcp::tool(
        description = "List attachments for one ONES issue and return temporary download URLs"
    )]
    async fn list_issue_attachments(
        &self,
        Parameters(IssueAttachmentsRequest { issue_uuid, cursor }): Parameters<
            IssueAttachmentsRequest,
        >,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<AttachmentsResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let attachments = map_error(
            self.service
                .list_issue_attachments(&auth, &issue_uuid, cursor.as_deref())
                .await,
        )?;
        Ok(Json(attachments))
    }

    #[rmcp::tool(
        description = "List ONES code links for one issue, including branches, commits, and pull requests"
    )]
    async fn list_issue_code_links(
        &self,
        Parameters(request): Parameters<IssueCodeLinksRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<CodeLinksResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let links = map_error(
            self.service
                .list_issue_code_links(&auth, &request.issue_uuid, request.redact.unwrap_or(true))
                .await,
        )?;
        Ok(Json(links))
    }

    #[rmcp::tool(
        description = "Create a source-control branch through ONES for one issue and linked repository"
    )]
    async fn create_issue_branch(
        &self,
        Parameters(request): Parameters<CreateBranchRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<CreateBranchResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let result = map_error(
            self.service
                .create_issue_branch(&auth, &request.issue_uuid, &request.input)
                .await,
        )?;
        Ok(Json(result))
    }

    #[rmcp::tool(
        description = "List available ONES status transitions and their fields for one issue"
    )]
    async fn list_issue_transitions(
        &self,
        Parameters(IssueTransitionLookup { issue_uuid }): Parameters<IssueTransitionLookup>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<IssueTransitionsResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let transitions = map_error(
            self.service
                .list_issue_transitions(&auth, &issue_uuid)
                .await,
        )?;
        Ok(Json(transitions))
    }

    #[rmcp::tool(description = "Preview an ONES status transition without writing changes")]
    async fn preview_issue_transition(
        &self,
        Parameters(request): Parameters<IssueTransitionRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<TransitionPreviewResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let preview = map_error(
            self.service
                .preview_issue_transition(&auth, &request.issue_uuid, &request.values)
                .await,
        )?;
        Ok(Json(preview))
    }

    #[rmcp::tool(description = "Execute a previously previewed ONES status transition")]
    async fn execute_issue_transition(
        &self,
        Parameters(request): Parameters<IssueTransitionRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<TransitionExecutionResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let result = map_error(
            self.service
                .execute_issue_transition(&auth, &request.issue_uuid, &request.values)
                .await,
        )?;
        Ok(Json(result))
    }

    #[rmcp::tool(
        description = "List compact, human-readable activity for one ONES issue; use the returned cursor for the next page"
    )]
    async fn list_issue_activity(
        &self,
        Parameters(request): Parameters<ListIssueActivityRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ActivityResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let activity = map_error(
            self.service
                .list_issue_activity(
                    &auth,
                    &request.issue_uuid,
                    request.cursor.as_deref(),
                    request.limit,
                    request.redact.unwrap_or(true),
                )
                .await,
        )?;
        Ok(Json(activity))
    }

    #[rmcp::tool(
        description = "List recorded and estimated work hours for one ONES issue, optionally filtered by owner and date range"
    )]
    async fn list_issue_work_hours(
        &self,
        Parameters(query): Parameters<ListIssueWorkHoursQuery>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<IssueWorkHoursResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let response = map_error(self.service.list_issue_work_hours(&auth, &query).await)?;
        Ok(Json(response))
    }

    #[rmcp::tool(
        description = "Query actual work hours by issue for an inclusive daily date range, optionally filtered by owner or one issue"
    )]
    async fn query_work_hour_report(
        &self,
        Parameters(query): Parameters<WorkHourReportQuery>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<WorkHourReportResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let response = map_error(self.service.query_work_hour_report(&auth, &query).await)?;
        Ok(Json(response))
    }

    #[rmcp::tool(description = "Record actual work hours directly on one ONES issue")]
    async fn add_work_hour(
        &self,
        Parameters(input): Parameters<AddWorkHourInput>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<WorkHourMutationResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let response = map_error(self.service.add_work_hour(&auth, &input).await)?;
        Ok(Json(response))
    }

    #[rmcp::tool(description = "Set remaining work hours for one ONES issue and owner")]
    async fn update_remaining_hours(
        &self,
        Parameters(input): Parameters<UpdateRemainingHoursInput>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<WorkHourMutationResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let response = map_error(self.service.update_remaining_hours(&auth, &input).await)?;
        Ok(Json(response))
    }

    #[rmcp::tool(description = "Delete one recorded or estimated work-hour entry by key")]
    async fn delete_work_hour(
        &self,
        Parameters(input): Parameters<DeleteWorkHourInput>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<WorkHourMutationResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let response = map_error(self.service.delete_work_hour(&auth, &input).await)?;
        Ok(Json(response))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct WorkbencViewRequest {
    view_uuid: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListWorkbenchIssuesRequest {
    view_uuid: String,
    group_value: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
    redact: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DisplayIdRequest {
    display_id: String,
    redact: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IssueUuidRequest {
    issue_uuid: String,
    redact: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IssueAttachmentsRequest {
    issue_uuid: String,
    cursor: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IssueCodeLinksRequest {
    issue_uuid: String,
    redact: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CreateBranchRequest {
    issue_uuid: String,
    #[serde(flatten)]
    input: CreateBranchInput,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IssueTransitionLookup {
    issue_uuid: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IssueTransitionRequest {
    issue_uuid: String,
    #[serde(flatten)]
    values: TransitionValues,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListIssueActivityRequest {
    issue_uuid: String,
    cursor: Option<String>,
    limit: Option<u32>,
    redact: Option<bool>,
}

pub(super) fn map_error<T>(result: AppResult<T>) -> Result<T, rmcp::ErrorData> {
    result.map_err(|error| rmcp::ErrorData::internal_error(error.to_string(), None))
}
