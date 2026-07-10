use super::*;
use crate::ones::test_support::test_core_fields;

#[test]
fn builds_group_query_from_onesql() {
    let query = build_group_count_query(
        "uid(field005) NOT IN ( uid('N1Qg4thd') )",
        "field006",
        &[SortField {
            field: "field005".to_string(),
            order: "desc".to_string(),
        }],
        &test_core_fields(),
    );
    assert!(query.contains("group by field006"));
    assert!(query.contains("uid(field006.uuid as value, field006.name as label)"));
}

#[test]
fn list_query_requires_group_value_when_grouped() {
    let error = build_issue_list_query(
        "1 = 1",
        Some("field006"),
        None,
        None,
        20,
        &[],
        &test_core_fields(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("group_value"));
}

#[test]
fn default_sort_uses_catalog_create_time() {
    let query =
        build_issue_list_query("1 = 1", None, None, None, 20, &[], &test_core_fields()).unwrap();
    assert!(query.contains("order by created desc"), "{query}");
}

#[test]
fn detail_query_projects_core_and_valid_custom_fields() {
    let definitions: Vec<IssueFieldDefinition> = serde_json::from_value(serde_json::json!([
        { "uuid": "titleForProject", "name": "Title", "built_in": true, "type": { "uuid": "name", "value_type": "text" } },
        { "uuid": "assigneeForProject", "name": "Assignee", "built_in": true, "type": { "uuid": "assign", "value_type": "single_reference_object", "reference_object_type": "user" } },
        { "uuid": "field057", "name": "Comments", "built_in": true, "type": { "uuid": "comments", "value_type": "multi_reference_object" } },
        { "uuid": "Wy65kRnx", "name": "Business ID", "built_in": false, "type": { "value_type": "text" } },
        { "uuid": "BfyseLJz", "name": "Developer", "built_in": false, "type": { "value_type": "single_reference_object", "reference_object_type": "user" } },
        { "uuid": "K2VETsUz", "name": "Systems", "built_in": false, "type": { "value_type": "multi_reference_object", "reference_object_type": "option" } },
        { "uuid": "RBQvxzjH", "name": "Linked issues", "built_in": false, "type": { "value_type": "multi_reference_object", "reference_object_type": "issue" } },
        { "uuid": "invalid-field", "name": "Invalid", "built_in": false, "type": { "value_type": "text" } }
    ])).unwrap();
    let query = build_issue_detail_query("issue-1", &test_core_fields(), &definitions);
    assert!(!query.contains("titleForProject"), "{query}");
    assert!(!query.contains("assigneeForProject"), "{query}");
    assert!(query.contains("assignee.uuid,assignee.name"), "{query}");
    assert!(query.contains("Wy65kRnx"), "{query}");
    assert!(query.contains("BfyseLJz.uuid,BfyseLJz.name"), "{query}");
    assert!(query.contains("K2VETsUz.uuid,K2VETsUz.name"), "{query}");
    assert!(!query.contains("field057"));
    assert!(!query.contains("RBQvxzjH"));
    assert!(!query.contains("invalid-field"));
}
