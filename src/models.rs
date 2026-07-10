use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthContext {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkbenchViewSummary {
    pub uuid: String,
    pub name: String,
    pub shared: bool,
    pub built_in: bool,
    pub is_show: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SortField {
    pub field: String,
    pub order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkbenchViewDetail {
    pub uuid: String,
    pub name: String,
    pub group_by: Option<String>,
    pub layout: Option<String>,
    pub display_type: Option<String>,
    pub shared: bool,
    pub built_in: bool,
    pub include_subtasks: bool,
    pub is_fold_all_groups: bool,
    pub sort: Vec<SortField>,
    pub hierarchy_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkbenchIssueGroup {
    pub value: Option<String>,
    pub label: String,
    pub total_count: u64,
    pub hierarchy_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub page_count: Option<u64>,
    pub total_count: Option<u64>,
    pub start_pos: Option<u64>,
    pub end_pos: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NamedRef {
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueLookupResult {
    pub issue_uuid: String,
    pub display_id: Option<String>,
    pub identifier: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueSummary {
    pub uuid: String,
    pub path: Option<String>,
    pub display_id: Option<String>,
    pub title: Option<String>,
    pub issue_type: Option<NamedRef>,
    pub status: Option<NamedRef>,
    pub project: Option<NamedRef>,
    pub assignees: Vec<NamedRef>,
    pub due_date: Option<String>,
    pub has_manhour_tab: Option<bool>,
    pub downstream_task_count: Option<u64>,
    pub downstream_task_done_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueCoreFields {
    pub creator: Option<NamedRef>,
    pub assignee: Option<NamedRef>,
    pub watchers: Vec<NamedRef>,
    pub priority: Option<NamedRef>,
    pub sprint: Option<NamedRef>,
    pub modules: Vec<NamedRef>,
    pub due_date: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayField {
    pub name: String,
    pub value: String,
    pub field_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueActivityEntry {
    pub entry_type: Option<String>,
    pub actor: Option<String>,
    pub time: Option<String>,
    pub summary: Option<String>,
    pub text: Option<String>,
    pub field_name: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActivityPageInfo {
    pub has_next_page: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueActivityPage {
    pub activity: Vec<IssueActivityEntry>,
    pub page_info: ActivityPageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelatedWikiPage {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub reference_type: Option<String>,
    pub sub_reference_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelatedCase {
    pub uuid: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueRelatedSummary {
    pub can_edit: Option<bool>,
    pub sprint: Option<NamedRef>,
    pub related_wiki_pages_count: Option<u64>,
    pub related_wiki_pages: Vec<RelatedWikiPage>,
    pub related_cases: Vec<RelatedCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueDetail {
    pub lookup: IssueLookupResult,
    pub summary: IssueSummary,
    pub core_fields: IssueCoreFields,
    pub custom_fields: Vec<DisplayField>,
    pub related: IssueRelatedSummary,
}
