use super::super::OnesService;
use super::{
    IssueContext,
    helpers::{is_identifier, value_uuid},
};
use crate::{
    error::{AppError, AppResult},
    models::AuthContext,
    ones::raw::IssueFieldDefinition,
};
use serde_json::{Value, json};
use std::collections::HashMap;

impl OnesService {
    pub(super) async fn issue_context(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<IssueContext> {
        let catalog = self.team_field_catalog(auth).await?;
        let fields = [
            catalog.core.project.as_str(),
            catalog.core.issue_type.as_str(),
        ];
        let item = self.current_values_item(auth, issue_uuid, &fields).await?;
        Ok(IssueContext {
            project_uuid: value_uuid(item.get(&catalog.core.project)),
            issue_type_uuid: value_uuid(item.get(&catalog.core.issue_type)),
        })
    }

    pub(super) async fn current_values(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        fields: &[IssueFieldDefinition],
    ) -> AppResult<HashMap<String, Value>> {
        if fields.is_empty() {
            return Ok(HashMap::new());
        }
        let ids = fields
            .iter()
            .map(|field| field.uuid.as_str())
            .collect::<Vec<_>>();
        self.current_values_item(auth, issue_uuid, &ids).await
    }

    async fn current_values_item(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        fields: &[&str],
    ) -> AppResult<HashMap<String, Value>> {
        let fields = fields
            .iter()
            .filter(|field| is_identifier(field))
            .copied()
            .collect::<Vec<_>>();
        if fields.is_empty() {
            return Ok(HashMap::new());
        }
        let query = format!(
            "select uid({}) from issue where uid(uuid) = uid('{}');",
            fields.join(","),
            issue_uuid.replace('\'', "''")
        );
        let response = self
            .client
            .workitems_onesql(auth, &json!({"query": query}))
            .await?;
        let item = response
            .data
            .into_iter()
            .find_map(|entry| entry.get("item").cloned())
            .ok_or_else(|| AppError::NotFound(format!("issue {issue_uuid}")))?;
        Ok(item
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect())
    }
}
