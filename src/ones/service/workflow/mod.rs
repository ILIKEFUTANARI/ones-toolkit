mod context;
mod helpers;
mod poll;
mod prepare;
mod time;
mod values;

use super::OnesService;
use crate::{
    error::{AppError, AppResult},
    ones::raw::IssueFieldDefinition,
    workflow::{
        IssueTransitionsResponse, TransitionExecutionResponse, TransitionFieldValue,
        TransitionPreviewResponse, TransitionSummary, TransitionValues,
    },
};
use helpers::{apply_rule_states, transition_summary};
use serde_json::{Value, json};
use std::collections::HashMap;

struct IssueContext {
    project_uuid: String,
    issue_type_uuid: String,
}

struct PreparedTransition {
    summary: TransitionSummary,
    fields: Vec<IssueFieldDefinition>,
    context: IssueContext,
    current_values: HashMap<String, Value>,
}

struct ValidationResult {
    prepared: PreparedTransition,
    normalized: Vec<TransitionFieldValue>,
    missing_required_fields: Vec<String>,
    permission_errors: Vec<String>,
    validation_errors: Vec<String>,
}

impl OnesService {
    pub async fn list_issue_transitions(
        &self,
        auth: &crate::models::AuthContext,
        issue_uuid: &str,
    ) -> AppResult<IssueTransitionsResponse> {
        let context = self.issue_context(auth, issue_uuid).await?;
        let records = self.client.issue_transitions(auth, issue_uuid).await?;
        let mut transitions = Vec::with_capacity(records.transitions.len());
        for record in records.transitions {
            let detail = self
                .client
                .transition_detail(auth, &record.uuid, &context.issue_type_uuid)
                .await?;
            let form_fields = self.client.transition_fields(auth, &record.uuid).await?;
            let mut summary = transition_summary(&record, &detail, form_fields);
            if !summary.fields.is_empty() {
                let rules = self
                    .client
                    .execute_transition_form_rule(auth, issue_uuid, &record.uuid, &[])
                    .await?;
                apply_rule_states(&mut summary, &rules.widget_states);
            }
            transitions.push(summary);
        }
        Ok(IssueTransitionsResponse {
            issue_uuid: issue_uuid.to_string(),
            transitions,
        })
    }

    pub async fn preview_issue_transition(
        &self,
        auth: &crate::models::AuthContext,
        issue_uuid: &str,
        values: &TransitionValues,
    ) -> AppResult<TransitionPreviewResponse> {
        let result = self.validate_transition(auth, issue_uuid, values).await?;
        Ok(TransitionPreviewResponse {
            issue_uuid: issue_uuid.to_string(),
            transition: result.prepared.summary,
            valid: result.missing_required_fields.is_empty()
                && result.permission_errors.is_empty()
                && result.validation_errors.is_empty(),
            normalized_field_values: result.normalized,
            missing_required_fields: result.missing_required_fields,
            permission_errors: result.permission_errors,
            validation_errors: result.validation_errors,
        })
    }

    pub async fn execute_issue_transition(
        &self,
        auth: &crate::models::AuthContext,
        issue_uuid: &str,
        values: &TransitionValues,
    ) -> AppResult<TransitionExecutionResponse> {
        let result = self.validate_transition(auth, issue_uuid, values).await?;
        let errors = result
            .missing_required_fields
            .iter()
            .chain(result.permission_errors.iter())
            .chain(result.validation_errors.iter())
            .cloned()
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            return Err(AppError::invalid_input(errors.join("; ")));
        }

        let field_values = result
            .normalized
            .iter()
            .map(|value| json!({"field_uuid": value.field_uuid, "value": value.value}))
            .collect::<Vec<_>>();
        let async_tokens = self
            .client
            .new_transition(auth, issue_uuid, &values.transition_uuid, &field_values)
            .await?;
        poll::wait_for_transition(
            self,
            auth,
            &async_tokens.async_token,
            async_tokens.async_compute_token.as_deref(),
        )
        .await?;

        Ok(TransitionExecutionResponse {
            issue_uuid: issue_uuid.to_string(),
            transition_uuid: values.transition_uuid.clone(),
            target_status: result.prepared.summary.to_status,
        })
    }
}
