use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(any(feature = "http", feature = "mcp"))]
use std::collections::BTreeMap;

use crate::models::StatusSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TransitionFieldValue {
    pub field_uuid: String,
    #[cfg_attr(feature = "mcp", schemars(with = "TransitionFieldValueSchema"))]
    #[cfg_attr(feature = "http", schema(value_type = TransitionFieldValueSchema))]
    pub value: Value,
}

#[cfg(any(feature = "http", feature = "mcp"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum TransitionFieldValueSchema {
    String(String),
    Object(BTreeMap<String, Value>),
    ObjectArray(Vec<BTreeMap<String, Value>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct WorklogInput {
    pub hours: f64,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TransitionValues {
    pub transition_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worklog: Option<WorklogInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_values: Vec<TransitionFieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TransitionSummary {
    pub uuid: String,
    pub name: String,
    pub from_status: StatusSummary,
    pub to_status: StatusSummary,
    pub fields: Vec<TransitionField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TransitionField {
    pub uuid: String,
    pub name: String,
    pub type_uuid: String,
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_object_type: Option<String>,
    pub required: bool,
    pub visible: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct IssueTransitionsResponse {
    pub issue_uuid: String,
    pub transitions: Vec<TransitionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TransitionPreviewResponse {
    pub issue_uuid: String,
    pub transition: TransitionSummary,
    pub valid: bool,
    pub normalized_field_values: Vec<TransitionFieldValue>,
    pub missing_required_fields: Vec<String>,
    pub permission_errors: Vec<String>,
    pub validation_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct TransitionExecutionResponse {
    pub issue_uuid: String,
    pub transition_uuid: String,
    pub target_status: StatusSummary,
}

#[cfg(test)]
mod tests {
    use super::TransitionValues;

    #[test]
    fn preserves_supported_dynamic_field_value_shapes() {
        for value in [
            serde_json::json!("comment-demo-text"),
            serde_json::json!({"uuid": "comment-demo-001", "content": "comment"}),
            serde_json::json!([{"uuid": "comment-demo-001", "content": "comment"}]),
        ] {
            let input = serde_json::json!({
                "transition_uuid": "transition-demo-001",
                "field_values": [{
                    "field_uuid": "field057",
                    "value": value
                }]
            });
            let parsed: TransitionValues = serde_json::from_value(input.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), input);
        }
    }
}
