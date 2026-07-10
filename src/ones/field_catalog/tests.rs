use super::*;
use crate::ones::raw::IssueFormFieldsResponse;

fn team_fields() -> Vec<IssueFieldDefinition> {
    serde_json::from_str::<IssueFormFieldsResponse>(include_str!(
        "../../../tests/fixtures/team_fields.json"
    ))
    .unwrap()
    .fields
}

#[test]
fn resolves_fixed_core_and_parent_child_fields() {
    let catalog = FieldCatalog::new(team_fields()).unwrap();
    assert_eq!(catalog.core.assignee, "ownerDynamic");
    assert_eq!(catalog.core.due_date, "dueDynamic");
    assert_eq!(catalog.core.parent.as_deref(), Some("parentDynamic"));
    assert_eq!(catalog.core.children.as_deref(), Some("childrenDynamic"));
}

#[test]
fn rejects_ambiguous_core_fields() {
    let mut fields = team_fields();
    fields
        .iter_mut()
        .find(|field| field.uuid == "dueDynamic")
        .unwrap()
        .fixed = false;
    let error = FieldCatalog::new(fields).unwrap_err();
    assert!(error.to_string().contains("core type date"));
    assert!(error.to_string().contains("2 candidates"));
}

#[test]
fn rejects_missing_core_fields() {
    let mut fields = team_fields();
    fields.retain(|field| field.uuid != "ownerDynamic");
    let error = FieldCatalog::new(fields).unwrap_err();
    assert!(error.to_string().contains("core type assign"));
    assert!(error.to_string().contains("0 candidates"));
}

#[test]
fn validates_reference_group_fields() {
    let catalog = FieldCatalog::new(team_fields()).unwrap();
    catalog.validate_group_field("projectDynamic").unwrap();
    let error = catalog.validate_group_field("releaseDynamic").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("not a groupable reference field")
    );
}
