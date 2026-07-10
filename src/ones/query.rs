use crate::error::{AppError, AppResult};
use crate::models::SortField;
use crate::ones::{
    field::{DisplayFieldKind, display_field_kind},
    field_catalog::CoreFields,
    raw::IssueFieldDefinition,
};
use std::collections::HashSet;

pub const DEFAULT_PAGE_LIMIT: u32 = 20;
pub const MAX_PAGE_LIMIT: u32 = 100;
pub const GROUP_PAGE_LIMIT: u32 = 2000;

pub(crate) fn normalize_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_PAGE_LIMIT).clamp(1, MAX_PAGE_LIMIT)
}

pub(crate) fn build_group_count_query(
    onesql: &str,
    group_by: &str,
    sort: &[SortField],
    core: &CoreFields,
) -> String {
    format!(
        "select count() as total_count, uid({group_by}.uuid as value, {group_by}.name as label) from issue where {onesql} group by {group_by} order by {} limit {GROUP_PAGE_LIMIT}",
        build_order_clause(sort, core)
    )
}

pub(crate) fn build_issue_list_query(
    onesql: &str,
    group_by: Option<&str>,
    group_value: Option<&str>,
    cursor: Option<&str>,
    limit: u32,
    sort: &[SortField],
    core: &CoreFields,
) -> AppResult<String> {
    let group_clause = match (group_by, group_value) {
        (Some(field), Some(value)) => {
            format!(
                "(({}) AND (uid({field}) IN (uid('{}'))))",
                onesql,
                escape(value)
            )
        }
        (Some(_), None) => {
            return Err(AppError::invalid_input(
                "group_value is required for grouped workbench views",
            ));
        }
        (None, Some(_)) => {
            return Err(AppError::invalid_input(
                "group_value is not supported for ungrouped workbench views",
            ));
        }
        (None, None) => format!("({onesql})"),
    };

    Ok(format!(
        "select uid(uuid,{title},{assignee}.uuid,{assignee}.name,{issue_type}.uuid,{issue_type}.name,{due_date},{status}.name,{status}.category,{project}.uuid,{project}.name),v$display_id_path as display_id_path,v$sub_workitem_count as downstream_task_count,v$sub_workitem_done_count as downstream_task_done_count from issue where ({group_clause}) AND (v$cursor > '{}') order by {} limit {limit}, {limit}",
        escape(cursor.unwrap_or("")),
        build_order_clause(sort, core),
        title = core.title,
        assignee = core.assignee,
        issue_type = core.issue_type,
        due_date = core.due_date,
        status = core.status,
        project = core.project,
    ))
}

pub(crate) fn build_issue_detail_query(
    issue_uuid: &str,
    core: &CoreFields,
    definitions: &[IssueFieldDefinition],
) -> String {
    let mut seen = HashSet::new();
    let fields = definitions
        .iter()
        .filter(|definition| seen.insert(definition.uuid.as_str()))
        .filter_map(|definition| {
            Some(match display_field_kind(definition)? {
                DisplayFieldKind::Scalar => definition.uuid.clone(),
                DisplayFieldKind::Reference => {
                    format!("{0}.uuid,{0}.name", definition.uuid)
                }
            })
        });
    let mut projection = core_projection(core);
    for field in fields {
        projection.push(',');
        projection.push_str(&field);
    }

    format!(
        "select uid({projection}) from issue where uid(uuid) = uid('{}');",
        escape(issue_uuid)
    )
}

pub(crate) fn parse_sort_fields(
    raw_sort: &[std::collections::HashMap<String, crate::ones::raw::SortDescriptor>],
) -> Vec<SortField> {
    raw_sort
        .iter()
        .flat_map(|item| {
            item.iter().map(|(field, descriptor)| SortField {
                field: field
                    .strip_prefix("field_values.")
                    .unwrap_or(field)
                    .to_string(),
                order: descriptor.order.clone(),
            })
        })
        .collect()
}

fn build_order_clause(sort: &[SortField], core: &CoreFields) -> String {
    if sort.is_empty() {
        return format!("{} desc", core.created_at);
    }

    sort.iter()
        .map(|item| format!("{} {}", item.field, item.order))
        .collect::<Vec<_>>()
        .join(", ")
}

fn core_projection(core: &CoreFields) -> String {
    format!(
        "uuid,{display_id},{title},{issue_type}.uuid,{issue_type}.name,{status}.uuid,{status}.name,{status}.category,{project}.uuid,{project}.name,{creator}.uuid,{creator}.name,{assignee}.uuid,{assignee}.name,{watchers}.uuid,{watchers}.name,{priority}.uuid,{priority}.name,{sprint}.uuid,{sprint}.name,{modules}.uuid,{modules}.name,{due_date},{created_at},{updated_at},{description}",
        display_id = core.display_id,
        title = core.title,
        issue_type = core.issue_type,
        status = core.status,
        project = core.project,
        creator = core.creator,
        assignee = core.assignee,
        watchers = core.watchers,
        priority = core.priority,
        sprint = core.sprint,
        modules = core.modules,
        due_date = core.due_date,
        created_at = core.created_at,
        updated_at = core.updated_at,
        description = core.description,
    )
}

fn escape(input: &str) -> String {
    input.replace('\'', "''")
}

#[cfg(test)]
mod tests;
