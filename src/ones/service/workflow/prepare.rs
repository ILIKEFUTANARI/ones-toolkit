use super::super::OnesService;
use super::{
    PreparedTransition, ValidationResult,
    helpers::{apply_rule_states, field_name, has_value, merge_fields, transition_summary},
};
use crate::{
    error::{AppError, AppResult},
    models::AuthContext,
    workflow::TransitionValues,
};
use serde_json::json;
use std::collections::{HashMap, HashSet};

impl OnesService {
    pub(super) async fn validate_transition(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        values: &TransitionValues,
    ) -> AppResult<ValidationResult> {
        let mut prepared = self
            .prepare_transition(auth, issue_uuid, &values.transition_uuid)
            .await?;
        let mut validation_errors = Vec::new();
        let normalized = self
            .normalize_values(auth, issue_uuid, values, &prepared, &mut validation_errors)
            .await?;
        let known_fields = prepared
            .fields
            .iter()
            .map(|field| field.uuid.as_str())
            .collect::<HashSet<_>>();
        let submitted = normalized
            .iter()
            .filter(|value| known_fields.contains(value.field_uuid.as_str()))
            .map(|value| value.field_uuid.clone())
            .collect::<HashSet<_>>();
        let field_values = normalized
            .iter()
            .filter(|value| known_fields.contains(value.field_uuid.as_str()))
            .map(|value| json!({"field_uuid": value.field_uuid, "value": value.value}))
            .collect::<Vec<_>>();
        let rules = self
            .client
            .execute_transition_form_rule(auth, issue_uuid, &values.transition_uuid, &field_values)
            .await?;
        apply_rule_states(&mut prepared.summary, &rules.widget_states);
        let dynamic_states = rules
            .widget_states
            .into_iter()
            .map(|state| (state.field_uuid.clone(), state))
            .collect::<HashMap<_, _>>();
        let missing_required_fields = prepared
            .fields
            .iter()
            .filter(|field| {
                let state = dynamic_states.get(&field.uuid);
                state
                    .map(|state| state.visible)
                    .unwrap_or(field.visible.unwrap_or(true))
                    && state
                        .map(|state| state.required)
                        .unwrap_or(field.required.unwrap_or(false))
            })
            .filter(|field| {
                let supplied = normalized
                    .iter()
                    .find(|value| value.field_uuid == field.uuid)
                    .map(|value| has_value(Some(&value.value)))
                    .unwrap_or(false);
                !supplied && !has_value(prepared.current_values.get(&field.uuid))
            })
            .filter_map(|field| field_name(&prepared.fields, &field.uuid))
            .collect::<Vec<_>>();
        let permission_errors = self.permission_errors(auth, issue_uuid, &submitted).await?;
        Ok(ValidationResult {
            prepared,
            normalized,
            missing_required_fields,
            permission_errors,
            validation_errors,
        })
    }

    async fn prepare_transition(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        transition_uuid: &str,
    ) -> AppResult<PreparedTransition> {
        let context = self.issue_context(auth, issue_uuid).await?;
        let records = self.client.issue_transitions(auth, issue_uuid).await?;
        let record = records
            .transitions
            .into_iter()
            .find(|record| record.uuid == transition_uuid)
            .ok_or_else(|| AppError::NotFound(format!("transition {transition_uuid}")))?;
        let detail = self
            .client
            .transition_detail(auth, transition_uuid, &context.issue_type_uuid)
            .await?;
        let form_fields = self.client.transition_fields(auth, transition_uuid).await?;
        let fields = merge_fields(record.fields.clone(), form_fields);
        let current_values = self.current_values(auth, issue_uuid, &fields).await?;
        let summary = transition_summary(&record, &detail, fields.clone());
        Ok(PreparedTransition {
            summary,
            fields,
            context,
            current_values,
        })
    }

    async fn permission_errors(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        submitted: &HashSet<String>,
    ) -> AppResult<Vec<String>> {
        let fields = submitted.iter().cloned().collect::<Vec<_>>();
        let response = self
            .client
            .check_update_permission(auth, issue_uuid, &fields)
            .await?;
        let Some(permission) = response.0.first() else {
            return Ok(vec![
                "ONES returned no update permission result".to_string(),
            ]);
        };
        let mut errors = Vec::new();
        if !permission.has_update_permission {
            errors.push("issue update permission denied".to_string());
        }
        for field in submitted {
            if permission.field_permissions.get(field) == Some(&false) {
                errors.push(format!("field {field} update permission denied"));
            }
        }
        Ok(errors)
    }
}
