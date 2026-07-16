use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthContext {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkbenchViewSummary {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkbenchViewDetail {
    pub uuid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    pub include_subtasks: bool,
    pub hierarchy_enabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct SortField {
    pub field: String,
    pub order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkbenchIssueGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub label: String,
    pub total_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchy_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, ToSchema)]
pub struct PageInfo {
    pub has_next_page: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct IssueLookupResult {
    pub issue_uuid: String,
    pub display_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct StatusSummary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueListItem {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StatusSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downstream_task_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downstream_task_done_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct DisplayField {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct RelatedWikiPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct RelatedCase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueDetail {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StatusSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<DisplayField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_wiki_pages: Option<Vec<RelatedWikiPage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_cases: Option<Vec<RelatedCase>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueActivityEntry {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueActivityPage {
    pub activity: Vec<IssueActivityEntry>,
    pub page_info: PageInfo,
}
