use super::super::OnesService;
use super::{ONES_HOURS_UNIT, PreparedTransition};
use super::{helpers::semantic_field, time::parse_start_time};
use crate::{
    error::AppResult,
    models::AuthContext,
    ones::raw::IssueFieldDefinition,
    workflow::{TransitionFieldValue, TransitionValues},
};
use serde_json::{Value, json};
use std::collections::HashSet;

impl OnesService {
    pub(super) async fn normalize_values(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        values: &TransitionValues,
        prepared: &PreparedTransition,
        errors: &mut Vec<String>,
    ) -> AppResult<Vec<TransitionFieldValue>> {
        let mut seen = HashSet::new();
        let mut normalized = Vec::new();
        for value in &values.field_values {
            let field = prepared
                .fields
                .iter()
                .find(|field| field.uuid == value.field_uuid);
            if let Some(field) = field {
                validate_field_value(field, &value.value, errors);
            } else {
                errors.push(format!(
                    "field {} is not part of transition",
                    value.field_uuid
                ));
            }
            if !seen.insert(value.field_uuid.clone()) {
                errors.push(format!(
                    "field {} was supplied more than once",
                    value.field_uuid
                ));
            }
            normalized.push(value.clone());
        }

        if let Some(worklog) = &values.worklog {
            let field = semantic_field(&prepared.fields, "worklog", errors);
            if let Some(field) = field {
                if !seen.insert(field.uuid.clone()) {
                    errors.push(format!("field {} was supplied more than once", field.uuid));
                } else {
                    let valid_hours = worklog.hours.is_finite() && worklog.hours > 0.0;
                    if !valid_hours {
                        errors.push("worklog.hours must be a positive finite number".to_string());
                    }
                    let valid_description = !worklog.description.trim().is_empty();
                    if !valid_description {
                        errors.push("worklog.description must be non-empty".to_string());
                    }
                    let start_time = match parse_start_time(worklog.start_at.as_deref()) {
                        Ok(start_time) => Some(start_time),
                        Err(error) => {
                            errors.push(error.to_string());
                            None
                        }
                    };
                    if valid_hours
                        && valid_description
                        && let Some(start_time) = start_time
                    {
                        let units = (worklog.hours * ONES_HOURS_UNIT).round() as i64;
                        if units <= 0 {
                            errors.push("worklog.hours is below the ONES precision".to_string());
                        }
                        let owner = self.client.current_user_uuid(auth).await?;
                        if prepared.context.project_uuid.is_empty() {
                            errors.push(
                                "issue project UUID is unavailable for worklog validation"
                                    .to_string(),
                            );
                        } else if units > 0 {
                            let limit = self
                                .client
                                .recordable_manhour(
                                    auth,
                                    &prepared.context.project_uuid,
                                    start_time,
                                    &owner,
                                )
                                .await?
                                .limit;
                            if units > limit {
                                errors.push(format!("worklog exceeds recordable limit {limit}"));
                            }
                            normalized.push(TransitionFieldValue {
                                field_uuid: field.uuid.clone(),
                                value: json!([{
                                    "description": worklog.description,
                                    "hours": units,
                                    "owner": owner,
                                    "startTime": start_time * 1000,
                                    "task": issue_uuid,
                                    "type": "recorded",
                                    "start_time": start_time,
                                }]),
                            });
                        }
                    }
                }
            }
        }

        if let Some(test_scope) = &values.test_scope {
            let field = semantic_field(&prepared.fields, "test_scope", errors);
            if let Some(field) = field {
                if !seen.insert(field.uuid.clone()) {
                    errors.push(format!("field {} was supplied more than once", field.uuid));
                } else if test_scope.trim().is_empty() {
                    errors.push("test_scope must be non-empty".to_string());
                } else {
                    normalized.push(TransitionFieldValue {
                        field_uuid: field.uuid.clone(),
                        value: Value::String(test_scope.clone()),
                    });
                }
            }
        }
        Ok(normalized)
    }
}

fn validate_field_value(field: &IssueFieldDefinition, value: &Value, errors: &mut Vec<String>) {
    if !has_value(value) {
        errors.push(format!("field {} must have a value", field.uuid));
        return;
    }
    let Some(field_type) = field.field_type.as_ref() else {
        return;
    };
    let value_type = field_type.value_type.as_deref().unwrap_or_default();
    let valid = match value_type {
        "text" | "multi_line_text" | "richtext" => value.is_string(),
        "integer" => {
            value.as_i64().is_some()
                || value.as_u64().is_some()
                || value.as_f64().is_some_and(|number| number.fract() == 0.0)
        }
        "float" => value.is_number(),
        "boolean" => value.is_boolean(),
        "single_reference_object" => reference_value(value),
        "multi_reference_object" => value
            .as_array()
            .is_some_and(|values| values.iter().all(reference_value)),
        "recorded_manhour_list" => value.is_array(),
        "duration_hour" | "duration_second" => value.is_number(),
        // ONES date fields are represented as either epoch values or strings by different APIs.
        "date" | "datetime" => value.is_number() || value.is_string(),
        _ => true,
    };
    if !valid {
        errors.push(format!(
            "field {} value does not match ONES type {}",
            field.uuid, value_type
        ));
    }
}

fn has_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        _ => true,
    }
}

fn reference_value(value: &Value) -> bool {
    value.as_str().is_some_and(|value| !value.trim().is_empty())
        || value
            .as_object()
            .and_then(|object| object.get("uuid"))
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
}
