use crate::{
    models::StatusSummary,
    ones::{
        raw::IssueFieldDefinition,
        workflow_raw::WidgetState,
        workflow_raw::{TransitionDetailResponse, TransitionRecord, TransitionStatusResponse},
    },
    workflow::{TransitionField, TransitionSummary},
};
use serde_json::Value;

pub(super) fn transition_summary(
    record: &TransitionRecord,
    detail: &TransitionDetailResponse,
    form_fields: Vec<IssueFieldDefinition>,
) -> TransitionSummary {
    let fields = merge_fields(record.fields.clone(), form_fields)
        .into_iter()
        .map(public_transition_field)
        .collect();
    TransitionSummary {
        uuid: record.uuid.clone(),
        name: detail.name.clone(),
        from_status: status_summary(&detail.start_status),
        to_status: status_summary(&detail.end_status),
        fields,
    }
}

pub(super) fn apply_rule_states(summary: &mut TransitionSummary, states: &[WidgetState]) {
    for state in states {
        if let Some(field) = summary
            .fields
            .iter_mut()
            .find(|field| field.uuid == state.field_uuid)
        {
            field.required = state.required;
            field.visible = state.visible;
        }
    }
}

pub(super) fn merge_fields(
    mut transition_fields: Vec<IssueFieldDefinition>,
    form_fields: Vec<IssueFieldDefinition>,
) -> Vec<IssueFieldDefinition> {
    if transition_fields.is_empty() {
        return form_fields;
    }
    for form_field in form_fields {
        if let Some(field) = transition_fields
            .iter_mut()
            .find(|field| field.uuid == form_field.uuid)
        {
            field.name = form_field.name.or_else(|| field.name.clone());
            field.field_type = form_field.field_type.or_else(|| field.field_type.clone());
            field.required = form_field.required.or(field.required);
            field.visible = form_field.visible.or(field.visible);
            field.hidden = form_field.hidden.or(field.hidden);
        } else {
            transition_fields.push(form_field);
        }
    }
    transition_fields
}

fn public_transition_field(field: IssueFieldDefinition) -> TransitionField {
    let name = field.name.clone();
    let type_info = field
        .field_type
        .unwrap_or(crate::ones::raw::IssueFieldType {
            uuid: None,
            value_type: None,
            reference_object_type: None,
            can_be_filter: false,
            can_be_sort: false,
            can_be_group: false,
        });
    let type_uuid = type_info.uuid.unwrap_or_default();
    let value_type = type_info.value_type.unwrap_or_default();
    TransitionField {
        uuid: field.uuid,
        name: name.clone().unwrap_or_default(),
        semantic: semantic_for(&type_uuid, &value_type, name.as_deref()).map(str::to_string),
        type_uuid,
        value_type,
        reference_object_type: type_info.reference_object_type,
        required: field.required.unwrap_or(false),
        visible: field.visible.unwrap_or(true),
        hidden: field.hidden.unwrap_or(false),
    }
}

pub(super) fn semantic_field<'a>(
    fields: &'a [IssueFieldDefinition],
    semantic: &str,
    errors: &mut Vec<String>,
) -> Option<&'a IssueFieldDefinition> {
    let matches = fields
        .iter()
        .filter(|field| {
            semantic_for(
                field
                    .field_type
                    .as_ref()
                    .and_then(|info| info.uuid.as_deref())
                    .unwrap_or_default(),
                field
                    .field_type
                    .as_ref()
                    .and_then(|info| info.value_type.as_deref())
                    .unwrap_or_default(),
                field.name.as_deref(),
            ) == Some(semantic)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [field] => Some(field),
        [] => {
            errors.push(format!("no transition field matches semantic {semantic}"));
            None
        }
        _ => {
            errors.push(format!(
                "multiple transition fields match semantic {semantic}"
            ));
            None
        }
    }
}

fn semantic_for(type_uuid: &str, value_type: &str, name: Option<&str>) -> Option<&'static str> {
    if type_uuid == "recorded_manhour_list" {
        return Some("worklog");
    }
    let name = name.unwrap_or_default().to_lowercase();
    if matches!(value_type, "text" | "multi_line_text" | "richtext")
        && (name.contains("测试范围") || name.contains("test scope"))
    {
        return Some("test_scope");
    }
    None
}

pub(super) fn field_name(fields: &[IssueFieldDefinition], uuid: &str) -> Option<String> {
    fields
        .iter()
        .find(|field| field.uuid == uuid)
        .and_then(|field| field.name.clone())
        .or_else(|| Some(uuid.to_string()))
}

fn status_summary(status: &TransitionStatusResponse) -> StatusSummary {
    StatusSummary {
        name: status.name.clone(),
        category: status.category.clone(),
    }
}

pub(super) fn value_uuid(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Object(value)) => value
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}

pub(super) fn has_value(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) | None => false,
        Some(Value::String(value)) => !value.trim().is_empty(),
        Some(Value::Array(values)) => !values.is_empty(),
        _ => true,
    }
}

pub(super) fn is_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}
