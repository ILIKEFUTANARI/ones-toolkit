use crate::{
    error::{AppError, AppResult},
    ones::raw::IssueFieldDefinition,
};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct FieldCatalog {
    pub fields: Vec<IssueFieldDefinition>,
    pub core: CoreFields,
}

impl FieldCatalog {
    pub fn new(fields: Vec<IssueFieldDefinition>) -> AppResult<Self> {
        let core = CoreFields::resolve(&fields)?;
        Ok(Self { fields, core })
    }

    pub fn public_fields(&self) -> Vec<FieldDefinition> {
        public_definitions(&self.fields)
    }

    pub fn validate_group_field(&self, uuid: &str) -> AppResult<()> {
        let field = self
            .fields
            .iter()
            .find(|field| field.uuid == uuid)
            .ok_or_else(|| {
                AppError::invalid_upstream(format!(
                    "view group field {uuid} is absent from the team catalog"
                ))
            })?;
        let field_type = field.field_type.as_ref().ok_or_else(|| {
            AppError::invalid_upstream(format!("view group field {uuid} has no type metadata"))
        })?;
        if !field_type.can_be_group
            || !matches!(
                field_type.value_type.as_deref(),
                Some("single_reference_object" | "multi_reference_object")
            )
        {
            return Err(AppError::invalid_upstream(format!(
                "view group field {uuid} is not a groupable reference field"
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CoreFields {
    pub display_id: String,
    pub title: String,
    pub issue_type: String,
    pub status: String,
    pub project: String,
    pub creator: String,
    pub assignee: String,
    pub watchers: String,
    pub priority: String,
    pub sprint: String,
    pub modules: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub parent: Option<String>,
    pub children: Option<String>,
}

impl CoreFields {
    fn resolve(fields: &[IssueFieldDefinition]) -> AppResult<Self> {
        Ok(Self {
            display_id: resolve_core(fields, "display_id")?,
            title: resolve_core(fields, "name")?,
            issue_type: resolve_core(fields, "issue_type")?,
            status: resolve_core(fields, "status")?,
            project: resolve_core(fields, "project")?,
            creator: resolve_core(fields, "creator")?,
            assignee: resolve_core(fields, "assign")?,
            watchers: resolve_core(fields, "watcher")?,
            priority: resolve_core(fields, "priority")?,
            sprint: resolve_core(fields, "sprint")?,
            modules: resolve_core(fields, "multi_project_module")?,
            due_date: resolve_core(fields, "date")?,
            created_at: resolve_core(fields, "create_time")?,
            updated_at: resolve_core(fields, "update_time")?,
            description: resolve_core(fields, "rich_desc")?,
            parent: resolve_hierarchy(fields, "up")?,
            children: resolve_hierarchy(fields, "down")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct FieldDefinition {
    pub uuid: String,
    pub name: String,
    pub built_in: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed: Option<bool>,
    pub type_uuid: String,
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_object_type: Option<String>,
    pub can_filter: bool,
    pub can_sort: bool,
    pub can_group: bool,
}

impl FieldDefinition {
    fn from_raw(field: &IssueFieldDefinition) -> Option<Self> {
        let field_type = field.field_type.as_ref()?;
        Some(Self {
            uuid: field.uuid.clone(),
            name: field.name.clone().unwrap_or_default(),
            built_in: field.built_in,
            fixed: field.fixed.then_some(true),
            type_uuid: field_type.uuid.clone()?,
            value_type: field_type.value_type.clone()?,
            reference_object_type: field_type.reference_object_type.clone(),
            can_filter: field_type.can_be_filter,
            can_sort: field_type.can_be_sort,
            can_group: field_type.can_be_group,
        })
    }
}

pub fn public_definitions(fields: &[IssueFieldDefinition]) -> Vec<FieldDefinition> {
    fields
        .iter()
        .filter_map(FieldDefinition::from_raw)
        .collect()
}

fn resolve_core(fields: &[IssueFieldDefinition], type_uuid: &str) -> AppResult<String> {
    let candidates = fields
        .iter()
        .filter(|field| field.built_in && field_type(field) == Some(type_uuid))
        .collect::<Vec<_>>();
    let fixed = candidates
        .iter()
        .copied()
        .filter(|field| field.fixed)
        .collect::<Vec<_>>();
    let selected = match (candidates.as_slice(), fixed.as_slice()) {
        (_, [field]) => Some(*field),
        ([field], []) => Some(*field),
        _ => None,
    };
    selected.map(|field| field.uuid.clone()).ok_or_else(|| {
        AppError::invalid_upstream(format!(
            "team field catalog has ambiguous or missing core type {type_uuid}: {} candidates, {} fixed",
            candidates.len(),
            fixed.len()
        ))
    })
}

fn resolve_hierarchy(
    fields: &[IssueFieldDefinition],
    direction: &str,
) -> AppResult<Option<String>> {
    let candidates = fields
        .iter()
        .filter(|field| {
            field.built_in
                && field_type(field) == Some("hierarchy")
                && field
                    .setting
                    .as_ref()
                    .and_then(|setting| setting.config_key.as_deref())
                    == Some("parent_child")
                && field
                    .setting
                    .as_ref()
                    .and_then(|setting| setting.direction.as_deref())
                    == Some(direction)
        })
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [field] => Ok(Some(field.uuid.clone())),
        _ => Err(AppError::invalid_upstream(format!(
            "team field catalog has ambiguous parent_child {direction} fields: {} candidates",
            candidates.len()
        ))),
    }
}

fn field_type(field: &IssueFieldDefinition) -> Option<&str> {
    field.field_type.as_ref()?.uuid.as_deref()
}

#[cfg(test)]
mod tests;
