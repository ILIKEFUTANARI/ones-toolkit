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
    #[serde(default)]
    pub shared: bool,
    #[serde(default)]
    pub built_in: bool,
}

#[derive(Debug, Deserialize)]
pub struct ViewConfigResponse {
    pub configs: Vec<ViewConfigItem>,
}

#[derive(Debug, Deserialize)]
pub struct ViewConfigItem {
    pub view_uuid: String,
    #[serde(default)]
    pub is_show: bool,
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
    pub layout: Option<String>,
    pub display_type: Option<String>,
    #[serde(default)]
    pub shared: bool,
    #[serde(default)]
    pub built_in: bool,
    #[serde(default)]
    pub include_subtasks: bool,
    #[serde(default)]
    pub is_fold_all_groups: bool,
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
    pub identifier: Option<String>,
    pub display_id: Option<String>,
    pub display_id_path: Option<String>,
    pub task_uuid: Option<String>,
    pub path: Option<String>,
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
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub page_count: Option<u64>,
    pub total_count: Option<u64>,
    pub start_pos: Option<u64>,
    pub end_pos: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct IssueFormFieldsResponse {
    #[serde(default)]
    pub fields: Vec<Value>,
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
