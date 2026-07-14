use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub user: LoginUser,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub uuid: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ViewListResponse {
    pub views: Vec<ViewListItem>,
}

#[derive(Debug, Deserialize)]
pub struct ViewListItem {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ViewDetailResponse {
    pub views: Vec<ViewDetailItem>,
}

#[derive(Debug, Deserialize)]
pub struct ViewDetailItem {
    pub uuid: String,
    pub name: String,
    pub group_by: Option<String>,
    #[serde(default)]
    pub include_subtasks: bool,
    #[serde(default)]
    pub sort: Vec<HashMap<String, SortDescriptor>>,
    pub condition: Option<ViewCondition>,
    pub hierarchy_config: Option<ViewHierarchyConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SortDescriptor {
    pub order: String,
}

#[derive(Debug, Deserialize)]
pub struct ViewCondition {
    pub onesql: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ViewHierarchyConfig {
    #[serde(default)]
    pub enabled: bool,
    pub upstream_field_uuid: Option<String>,
    pub downstream_field_uuid: Option<String>,
    pub perspective: Option<bool>,
    pub tree_data: Option<TreeNode>,
}

#[derive(Debug, Deserialize)]
pub struct TreeNode {
    pub uuid: String,
    #[serde(default)]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Deserialize)]
pub struct IdentifierResponse {
    pub display_id: Option<String>,
    pub task_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OnesqlResponse {
    #[serde(default)]
    pub data: Vec<Value>,
    pub page_info: Option<PageInfoRaw>,
}

#[derive(Debug, Deserialize)]
pub struct PageInfoRaw {
    #[serde(default)]
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
    pub total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct IssueFormFieldsResponse {
    #[serde(default)]
    pub fields: Vec<IssueFieldDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueFieldDefinition {
    pub uuid: String,
    pub name: Option<String>,
    #[serde(default)]
    pub built_in: bool,
    #[serde(default)]
    pub fixed: bool,
    #[serde(rename = "type")]
    pub field_type: Option<IssueFieldType>,
    pub setting: Option<IssueFieldSetting>,
    pub required: Option<bool>,
    pub visible: Option<bool>,
    pub hidden: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueFieldType {
    pub uuid: Option<String>,
    pub value_type: Option<String>,
    pub reference_object_type: Option<String>,
    #[serde(default)]
    pub can_be_filter: bool,
    #[serde(default)]
    pub can_be_sort: bool,
    #[serde(default)]
    pub can_be_group: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueFieldSetting {
    pub config_key: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    #[serde(default)]
    pub messages: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct QuickActionResponse {
    pub data: QuickActionData,
}

#[derive(Debug, Deserialize)]
pub struct QuickActionData {
    pub task: Value,
}
