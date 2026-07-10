use crate::error::{AppError, AppResult};
use crate::models::SortField;
use crate::ones::{
    field::{DisplayFieldKind, display_field_kind},
    raw::IssueFieldDefinition,
};
use std::collections::HashSet;

pub const DEFAULT_PAGE_LIMIT: u32 = 20;
pub const MAX_PAGE_LIMIT: u32 = 100;
pub const GROUP_PAGE_LIMIT: u32 = 2000;

const ISSUE_DETAIL_PROJECTION: &str = "uuid,v$issue_path,field001,field003.uuid,field003.name,field004.uuid,field004.name,field005.uuid,field005.name,field005.category,field006.uuid,field006.name,field007.uuid,field007.name,field008.uuid,field008.name,field009,field010,field011.uuid,field011.name,field011.status_category,field012.uuid,field012.name,toDate(field013),field016,field054.uuid,field054.name,field903";

pub fn normalize_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(DEFAULT_PAGE_LIMIT).clamp(1, MAX_PAGE_LIMIT)
}

pub fn build_group_count_query(onesql: &str, group_by: &str, sort: &[SortField]) -> String {
    format!(
        "select count() as total_count, uid({group_by}.uuid as value, {group_by}.name as label) from issue where {onesql} group by {group_by} order by {} limit {GROUP_PAGE_LIMIT}",
        build_order_clause(sort)
    )
}

pub fn build_issue_list_query(
    onesql: &str,
    group_by: Option<&str>,
    group_value: Option<&str>,
    cursor: Option<&str>,
    limit: u32,
    sort: &[SortField],
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
        "select uid(uuid,uuid as path,field001,field007.uuid,field007.name,field007.icon,field007.icon_uuid,field007.manhour_aggregate_mode,field013,field005.name,field005.category,field006.uuid,field017,field018,field042,field020,field045),v$display_id_path as display_id_path,v$path as path,v$list_pin_field as list_pin_field,v$has_view_manhour_tab as has_manhour_tab,v$sub_workitem_count as downstream_task_count,v$sub_workitem_done_count as downstream_task_done_count from issue where ({group_clause}) AND (v$cursor > '{}') order by {} limit {limit}, {limit}",
        escape(cursor.unwrap_or("")),
        build_order_clause(sort),
    ))
}

pub fn build_issue_detail_query(issue_uuid: &str, definitions: &[IssueFieldDefinition]) -> String {
    let mut seen = HashSet::new();
    let custom_fields = definitions
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
    let mut projection = ISSUE_DETAIL_PROJECTION.to_string();
    for custom_field in custom_fields {
        projection.push(',');
        projection.push_str(&custom_field);
    }

    format!(
        "select uid({projection}) from issue where uid(uuid) = uid('{}');",
        escape(issue_uuid)
    )
}

pub fn parse_sort_fields(
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

pub fn build_order_clause(sort: &[SortField]) -> String {
    if sort.is_empty() {
        return "field009 desc".to_string();
    }

    sort.iter()
        .map(|item| format!("{} {}", item.field, item.order))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape(input: &str) -> String {
    input.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_group_query_from_onesql() {
        let query = build_group_count_query(
            "uid(field005) NOT IN ( uid('N1Qg4thd') )",
            "field006",
            &[SortField {
                field: "field005".to_string(),
                order: "desc".to_string(),
            }],
        );
        assert!(query.contains("group by field006"));
        assert!(query.contains("uid(field006.uuid as value, field006.name as label)"));
    }

    #[test]
    fn list_query_requires_group_value_when_grouped() {
        let error =
            build_issue_list_query("1 = 1", Some("field006"), None, None, 20, &[]).unwrap_err();
        assert!(error.to_string().contains("group_value"));
    }

    #[test]
    fn detail_query_projects_valid_custom_fields() {
        let definitions: Vec<IssueFieldDefinition> = serde_json::from_value(serde_json::json!([
            { "uuid": "field057", "name": "Comments", "built_in": true, "type": { "value_type": "multi_reference_object" } },
            { "uuid": "Wy65kRnx", "name": "Business ID", "built_in": false, "type": { "value_type": "text" } },
            { "uuid": "BfyseLJz", "name": "Developer", "built_in": false, "type": { "value_type": "single_reference_object", "reference_object_type": "user" } },
            { "uuid": "K2VETsUz", "name": "Systems", "built_in": false, "type": { "value_type": "multi_reference_object", "reference_object_type": "option" } },
            { "uuid": "RBQvxzjH", "name": "Linked issues", "built_in": false, "type": { "value_type": "multi_reference_object", "reference_object_type": "issue" } },
            { "uuid": "invalid-field", "name": "Invalid", "built_in": false, "type": { "value_type": "text" } }
        ])).unwrap();
        let query = build_issue_detail_query("issue-1", &definitions);
        assert!(query.contains("field903,Wy65kRnx"), "{query}");
        assert!(query.contains("BfyseLJz.uuid,BfyseLJz.name"), "{query}");
        assert!(query.contains("K2VETsUz.uuid,K2VETsUz.name"), "{query}");
        assert!(!query.contains("field057"));
        assert!(!query.contains("RBQvxzjH"));
        assert!(!query.contains("invalid-field"));
    }
}
