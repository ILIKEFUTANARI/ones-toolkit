use crate::{
    error::{AppError, AppResult},
    http_api::auth_from_headers,
    models::{
        AuthContext, IssueDetail, IssueSummary, PageInfo, WorkbenchIssueGroup, WorkbenchViewDetail,
        WorkbenchViewSummary,
    },
    ones::service::OnesService,
};
use http::request::Parts;
use rmcp::{
    Json, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    service::RequestContext,
    tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    ) -> Result<Json<WorkbenchIssuesResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let page = map_error(
            self.service
                .list_workbench_issues(
                    &auth,
                    &request.view_uuid,
                    request.group_value.as_deref(),
                    request.cursor.as_deref(),
                    request.limit,
                )
                .await,
        )?;
        Ok(Json(WorkbenchIssuesResponse {
            issues: page.issues,
            page_info: page.page_info,
        }))
    }

    #[rmcp::tool(
        description = "Resolve an ONES display ID like DEMO-1234 and return the issue detail"
    )]
    async fn get_issue_by_display_id(
        &self,
        Parameters(DisplayIdRequest { display_id }): Parameters<DisplayIdRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<IssueResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let issue = map_error(
            self.service
                .get_issue_by_display_id(&auth, &display_id)
                .await,
        )?;
        Ok(Json(IssueResponse { issue }))
    }

    #[rmcp::tool(description = "Return the aggregated issue detail for one ONES issue UUID")]
    async fn get_issue_detail(
        &self,
        Parameters(IssueUuidRequest { issue_uuid }): Parameters<IssueUuidRequest>,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<IssueResponse>, rmcp::ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let issue = map_error(self.service.get_issue_detail(&auth, &issue_uuid).await)?;
        Ok(Json(IssueResponse { issue }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OnesMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Read-only ONES issue wrapper exposing workbench views and issue details.",
            )
            .with_server_info(rmcp::model::Implementation::new(
                "ones-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
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
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DisplayIdRequest {
    display_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct IssueUuidRequest {
    issue_uuid: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ViewsResponse {
    views: Vec<WorkbenchViewSummary>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ViewResponse {
    view: WorkbenchViewDetail,
}

#[derive(Debug, Serialize, JsonSchema)]
struct GroupsResponse {
    groups: Vec<WorkbenchIssueGroup>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct WorkbenchIssuesResponse {
    issues: Vec<IssueSummary>,
    page_info: PageInfo,
}

#[derive(Debug, Serialize, JsonSchema)]
struct IssueResponse {
    issue: IssueDetail,
}

fn map_error<T>(result: AppResult<T>) -> Result<T, rmcp::ErrorData> {
    result.map_err(|error| rmcp::ErrorData::internal_error(error.to_string(), None))
}
