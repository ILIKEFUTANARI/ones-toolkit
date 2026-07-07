use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub icon: Option<i64>,
    pub icon_uuid: Option<String>,
    pub detail_type: Option<i64>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListPinField {
    pub field_uuid: String,
    pub name: String,
    pub value: String,
    pub bg_color: Option<String>,
    pub color: Option<String>,
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
    pub list_pin_fields: Vec<ListPinField>,
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueMessage {
    pub uuid: Option<String>,
    pub message_type: Option<String>,
    pub subject_name: Option<String>,
    pub action: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub object_name: Option<String>,
    pub send_time: Option<i64>,
    pub raw: Value,
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
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueDetail {
    pub lookup: IssueLookupResult,
    pub summary: IssueSummary,
    pub detail_item: Value,
    pub detail_form: Value,
    pub form_fields: Vec<Value>,
    pub messages: Vec<IssueMessage>,
    pub related: Option<IssueRelatedSummary>,
}
