use super::{OnesMcpServer, map_error};
use crate::api::FieldsResponse;
use rmcp::{
    ErrorData, ServerHandler,
    model::{
        ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents,
        ResourceTemplate, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool_handler,
};

const TEAM_FIELDS_URI: &str = "ones://fields";
const ISSUE_FIELDS_PREFIX: &str = "ones://issues/";
const ISSUE_FIELDS_SUFFIX: &str = "/fields";

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OnesMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(
            "ONES Toolkit is an unofficial third-party integration exposing workbench views, issue details, attachments, code links, field metadata, and explicit writes. It is not affiliated with or endorsed by ONES. For status changes, discover with list_issue_transitions, preview with preview_issue_transition, and execute only after valid=true; the service revalidates all fields and permissions before writing.",
        )
        .with_server_info(rmcp::model::Implementation::new(
            "ones-toolkit",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new(TEAM_FIELDS_URI, "ones-team-fields")
                .with_title("ONES team field catalog")
                .with_description("Field metadata used to resolve ONES query fields")
                .with_mime_type("application/json"),
        ]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            ResourceTemplate::new("ones://issues/{issue_uuid}/fields", "ones-issue-fields")
                .with_title("ONES issue field catalog")
                .with_description("Fields applicable to one ONES issue")
                .with_mime_type("application/json"),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let auth = map_error(self.auth_from_context(&context))?;
        let fields = if request.uri == TEAM_FIELDS_URI {
            map_error(self.service.list_team_fields(&auth).await)?
        } else if let Some(issue_uuid) = issue_uuid_from_uri(&request.uri) {
            map_error(self.service.list_issue_fields(&auth, issue_uuid).await)?
        } else {
            return Err(ErrorData::resource_not_found(
                "unknown ONES resource URI",
                None,
            ));
        };
        let text = serde_json::to_string(&FieldsResponse { fields })
            .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(text, request.uri).with_mime_type("application/json"),
        ]))
    }
}

fn issue_uuid_from_uri(uri: &str) -> Option<&str> {
    let issue_uuid = uri
        .strip_prefix(ISSUE_FIELDS_PREFIX)?
        .strip_suffix(ISSUE_FIELDS_SUFFIX)?;
    (!issue_uuid.is_empty()
        && issue_uuid
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_')))
    .then_some(issue_uuid)
}
