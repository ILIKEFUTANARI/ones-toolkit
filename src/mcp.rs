use crate::{
    api::{
        ActivityResponse, GroupsResponse, IssueResponse, ViewResponse, ViewsResponse,
        WorkbenchIssuePage,
    },
    error::{AppError, AppResult},
    http_api::auth_from_headers,
    models::AuthContext,
    ones::service::OnesService,
};
use http::request::Parts;
use rmcp::{
    Json,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    schemars::JsonSchema,
    service::RequestContext,
    tool_router,
};
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
struct ListIssueActivityRequest {
    issue_uuid: String,
    cursor: Option<String>,
    limit: Option<u32>,
    redact: Option<bool>,
}

pub(super) fn map_error<T>(result: AppResult<T>) -> Result<T, rmcp::ErrorData> {
    result.map_err(|error| rmcp::ErrorData::internal_error(error.to_string(), None))
}
