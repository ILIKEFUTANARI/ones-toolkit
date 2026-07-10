use super::value::{object_field, string_field};
use crate::{
    models::{IssueListItem, StatusSummary},
    ones::{field_catalog::CoreFields, identity::IdentityRegistry},
};
use serde_json::Value;

pub fn parse_issue_list_item(
    item: Value,
    core: &CoreFields,
    registry: &IdentityRegistry,
    redact: bool,
) -> IssueListItem {
    let mut identities = registry.scope(redact);
    IssueListItem {
        uuid: string_field(&item, "uuid").unwrap_or_default(),
        display_id: string_field(&item, "display_id_path"),
        title: string_field(&item, &core.title),
        issue_type: object_field(&item, &core.issue_type).and_then(|value| ref_name(&value)),
        status: object_field(&item, &core.status).and_then(|value| status_summary(&value)),
        project: object_field(&item, &core.project).and_then(|value| ref_name(&value)),
        assignee: object_field(&item, &core.assignee).and_then(|value| identities.person(&value)),
        due_date: string_field(&item, &core.due_date),
        downstream_task_count: item.get("downstream_task_count").and_then(Value::as_u64),
        downstream_task_done_count: item
            .get("downstream_task_done_count")
            .and_then(Value::as_u64),
    }
}

pub(super) fn ref_name(value: &Value) -> Option<String> {
    string_field(value, "name").filter(|name| !name.is_empty())
}

pub(super) fn status_summary(value: &Value) -> Option<StatusSummary> {
    Some(StatusSummary {
        name: ref_name(value)?,
        category: string_field(value, "category")
            .or_else(|| string_field(value, "status_category")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ones::test_support::test_core_fields;
    use serde_json::json;

    #[test]
    fn list_item_keeps_task_identifiers_and_status() {
        let item = json!({
            "uuid": "issue-1",
            "display_id_path": "DEMO-1",
            "status": { "name": "Open", "category": "to_do" }
        });
        let issue = parse_issue_list_item(
            item,
            &test_core_fields(),
            &IdentityRegistry::default(),
            true,
        );
        assert_eq!(issue.uuid, "issue-1");
        assert_eq!(issue.display_id.as_deref(), Some("DEMO-1"));
        assert_eq!(issue.status.unwrap().name, "Open");
    }
}
