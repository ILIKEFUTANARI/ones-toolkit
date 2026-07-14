use crate::ones::raw::IssueFieldDefinition;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct TransitionListResponse {
    #[serde(default)]
    pub transitions: Vec<TransitionRecord>,
}

#[derive(Debug, Deserialize)]
pub struct TransitionRecord {
    pub uuid: String,
    #[serde(default)]
    pub fields: Vec<IssueFieldDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct TransitionDetailResponse {
    pub name: String,
    pub start_status: TransitionStatusResponse,
    pub end_status: TransitionStatusResponse,
}

#[derive(Debug, Deserialize)]
pub struct TransitionStatusResponse {
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransitionFormFieldsResponse {
    #[serde(default)]
    pub fields: Vec<IssueFieldDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct TransitionRuleResponse {
    #[serde(default)]
    pub widget_states: Vec<WidgetState>,
}

#[derive(Debug, Deserialize)]
pub struct WidgetState {
    pub field_uuid: String,
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct PermissionResponse(pub Vec<PermissionRecord>);

#[derive(Debug, Deserialize)]
pub struct PermissionRecord {
    #[serde(default)]
    pub has_update_permission: bool,
    #[serde(default)]
    pub field_permissions: HashMap<String, bool>,
}

#[derive(Debug, Deserialize)]
pub struct RecordableManhourResponse {
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct NewTransitResponse {
    pub async_token: String,
    pub async_compute_token: String,
}

#[derive(Debug, Deserialize)]
pub struct AsyncStatusResponse {
    #[serde(default)]
    pub results: HashMap<String, AsyncResult>,
}

#[derive(Debug, Deserialize)]
pub struct AsyncResult {
    pub status: String,
    pub error: Option<String>,
}
