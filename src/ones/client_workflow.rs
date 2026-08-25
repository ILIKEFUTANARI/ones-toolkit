use super::{client::OnesClient, raw::IssueFieldDefinition};
use crate::{
    error::AppResult,
    models::AuthContext,
    ones::workflow_raw::{
        AsyncStatusResponse, NewTransitResponse, PermissionResponse, RecordableManhourResponse,
        TransitionDetailResponse, TransitionFormFieldsResponse, TransitionListResponse,
        TransitionRuleResponse,
    },
};
use reqwest::Method;
use serde_json::{Value, json};

impl OnesClient {
    pub async fn current_user_uuid(&self, auth: &AuthContext) -> AppResult<String> {
        Ok(self.ensure_session(auth).await?.user_uuid)
    }

    pub async fn issue_transitions(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<TransitionListResponse> {
        // Prefer public legacy `…/task/:taskUUID/transitions`; fall back to
        // internal `…/issue/:taskUUID/v2/transitions` on 404/405.
        self.request_json_with_legacy_fallback(
            Method::GET,
            auth,
            [
                self.project_team_url(&format!("task/{issue_uuid}/transitions")),
                self.ones_project_team_url(&format!("issue/{issue_uuid}/v2/transitions")),
            ],
            None::<&()>,
        )
        .await
    }

    pub async fn transition_fields(
        &self,
        auth: &AuthContext,
        transition_uuid: &str,
    ) -> AppResult<Vec<IssueFieldDefinition>> {
        let response: TransitionFormFieldsResponse = self
            .request_json(
                Method::POST,
                auth,
                self.ones_project_team_url("issue_form/fields"),
                Some(&json!({
                    "transition_context": {"transition_uuid": transition_uuid}
                })),
            )
            .await?;
        Ok(response.fields)
    }

    pub async fn transition_detail(
        &self,
        auth: &AuthContext,
        transition_uuid: &str,
        issue_type_uuid: &str,
    ) -> AppResult<TransitionDetailResponse> {
        self.request_json(
            Method::GET,
            auth,
            self.ones_project_team_url(&format!(
                "transition/{transition_uuid}?issue_type_uuid={issue_type_uuid}&from_global_issue_type=false"
            )),
            None::<&()>,
        )
        .await
    }

    pub async fn execute_transition_form_rule(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        transition_uuid: &str,
        field_values: &[Value],
    ) -> AppResult<TransitionRuleResponse> {
        self.request_json(
            Method::POST,
            auth,
            self.ones_project_team_url("issue_form/execute_form_rule"),
            Some(&json!({
                "form_uuid": transition_uuid,
                "issue_uuid": issue_uuid,
                "issue_field_values": field_values,
            })),
        )
        .await
    }

    pub async fn check_update_permission(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        field_uuids: &[String],
    ) -> AppResult<PermissionResponse> {
        self.request_json(
            Method::POST,
            auth,
            self.ones_project_team_url("tasks/check_update_permission"),
            Some(&json!({
                "task_uuids": [issue_uuid],
                "check_fields": field_uuids,
            })),
        )
        .await
    }

    pub async fn recordable_manhour(
        &self,
        auth: &AuthContext,
        project_uuid: &str,
        record_time: i64,
        owner_uuid: &str,
    ) -> AppResult<RecordableManhourResponse> {
        self.request_json(
            Method::POST,
            auth,
            self.project_team_url(&format!("project/{project_uuid}/recordable_manhour")),
            Some(&json!({
                "record_time": record_time,
                "owner": owner_uuid,
            })),
        )
        .await
    }

    pub async fn new_transition(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        transition_uuid: &str,
        field_values: &[Value],
    ) -> AppResult<NewTransitResponse> {
        self.request_json_once(
            Method::POST,
            auth,
            self.project_team_url(&format!("task/{issue_uuid}/new_transit")),
            Some(&json!({
                "transition_uuid": transition_uuid,
                "field_values": field_values,
            })),
        )
        .await
    }

    pub async fn async_status(
        &self,
        auth: &AuthContext,
        async_token: &str,
        async_compute_token: Option<&str>,
    ) -> AppResult<AsyncStatusResponse> {
        let mut tokens = json!({"async_token": async_token});
        if let Some(async_compute_token) = async_compute_token {
            tokens["async_compute_token"] = Value::String(async_compute_token.to_string());
        }
        self.request_json(
            Method::POST,
            auth,
            self.ones_project_team_url("async_status"),
            Some(&json!({
                "tokens": tokens,
                "block": true,
            })),
        )
        .await
    }
}
