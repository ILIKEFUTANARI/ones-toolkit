use super::value::{object_field, string_field};
use crate::{
    models::{IssueListItem, StatusSummary},
    ones::identity::IdentityRegistry,
};
use serde_json::Value;

pub fn parse_issue_list_item(
    item: Value,
    registry: &IdentityRegistry,
    redact: bool,
) -> IssueListItem {
    let mut identities = registry.scope(redact);
    IssueListItem {
        uuid: string_field(&item, "uuid").unwrap_or_default(),
        display_id: string_field(&item, "display_id_path")
            .or_else(|| string_field(&item, "field903")),
        title: string_field(&item, "field001"),
        issue_type: object_field(&item, "field007").and_then(|value| ref_name(&value)),
        status: object_field(&item, "field005").and_then(|value| status_summary(&value)),
        project: object_field(&item, "field006").and_then(|value| ref_name(&value)),
        assignee: object_field(&item, "field004").and_then(|value| identities.person(&value)),
        due_date: string_field(&item, "field013"),
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
    use serde_json::json;

    #[test]
    fn list_item_keeps_task_identifiers_and_status() {
        let item = json!({
            "uuid": "issue-1",
            "display_id_path": "DEMO-1",
            "field005": { "name": "Open", "category": "to_do" }
        });
        let issue = parse_issue_list_item(item, &IdentityRegistry::default(), true);
        assert_eq!(issue.uuid, "issue-1");
        assert_eq!(issue.display_id.as_deref(), Some("DEMO-1"));
        assert_eq!(issue.status.unwrap().name, "Open");
    }
}
