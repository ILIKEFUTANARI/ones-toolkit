use super::value::{
    array_field, display_value, format_timestamp, html_to_text, object_field, string_field,
};
use crate::{
    models::{DisplayField, IssueCoreFields, IssueSummary, NamedRef},
    ones::{field::display_field_kind, raw::IssueFieldDefinition},
};
use serde_json::Value;

pub fn parse_issue_summary(item: Value) -> IssueSummary {
    IssueSummary {
        uuid: string_field(&item, "uuid").unwrap_or_default(),
        path: string_field(&item, "path"),
        display_id: string_field(&item, "display_id_path")
            .or_else(|| string_field(&item, "field903")),
        title: string_field(&item, "field001"),
        issue_type: object_field(&item, "field007").map(parse_named_ref),
        status: object_field(&item, "field005").map(parse_named_ref),
        project: object_field(&item, "field006").map(parse_named_ref),
        assignees: array_field(&item, "field008")
            .into_iter()
            .map(parse_named_ref)
            .collect(),
        due_date: string_field(&item, "field013"),
        has_manhour_tab: item
            .get("has_manhour_tab")
            .and_then(Value::as_i64)
            .map(|value| value != 0),
        downstream_task_count: item.get("downstream_task_count").and_then(Value::as_u64),
        downstream_task_done_count: item
            .get("downstream_task_done_count")
            .and_then(Value::as_u64),
    }
}

pub fn parse_core_fields(item: &Value) -> IssueCoreFields {
    IssueCoreFields {
        creator: object_field(item, "field003").map(parse_named_ref),
        assignee: object_field(item, "field004").map(parse_named_ref),
        watchers: array_field(item, "field008")
            .into_iter()
            .map(parse_named_ref)
            .collect(),
        priority: object_field(item, "field012").map(parse_named_ref),
        sprint: object_field(item, "field011").map(parse_named_ref),
        modules: array_field(item, "field054")
            .into_iter()
            .map(parse_named_ref)
            .collect(),
        due_date: string_field(item, "field013"),
        created_at: item.get("field009").and_then(format_timestamp),
        updated_at: item.get("field010").and_then(format_timestamp),
        description: item
            .get("field016")
            .and_then(Value::as_str)
            .map(html_to_text)
            .filter(|value| !value.is_empty()),
    }
}

pub fn parse_custom_fields(
    item: &Value,
    definitions: &[IssueFieldDefinition],
) -> Vec<DisplayField> {
    definitions
        .iter()
        .filter_map(|definition| {
            display_field_kind(definition)?;
            let value_type = definition
                .field_type
                .as_ref()
                .and_then(|field_type| field_type.value_type.as_deref());
            let value = display_value(item.get(&definition.uuid)?, value_type)?;
            Some(DisplayField {
                name: definition
                    .name
                    .clone()
                    .unwrap_or_else(|| "unnamed field".to_string()),
                value,
                field_uuid: definition.uuid.clone(),
            })
        })
        .collect()
}

pub(super) fn parse_named_ref(value: Value) -> NamedRef {
    NamedRef {
        uuid: string_field(&value, "uuid"),
        name: string_field(&value, "name"),
        category: string_field(&value, "category")
            .or_else(|| string_field(&value, "status_category")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_issue_summary() {
        let item = json!({
            "uuid": "issue-demo-001",
            "display_id_path": "DEMO-1234",
            "field001": "Sample issue for integration testing",
            "field005": { "name": "Done", "category": "done", "uuid": "status-demo-done" },
            "field006": { "uuid": "proj-demo-001", "name": "Demo Project" },
            "field007": { "uuid": "type-demo-bug", "name": "Bug" },
            "field008": [{ "uuid": "user-demo-001", "name": "Alex Example" }]
        });
        let summary = parse_issue_summary(item);
        assert_eq!(summary.display_id.as_deref(), Some("DEMO-1234"));
        assert_eq!(
            summary.project.and_then(|project| project.name).as_deref(),
            Some("Demo Project")
        );
    }

    #[test]
    fn custom_fields_are_named_and_keep_duplicate_labels() {
        let item = json!({
            "first": { "name": "A" },
            "second": 7,
            "empty": null,
            "builtIn": "hidden",
            "linkedIssue": [{ "name": "hidden" }]
        });
        let definitions: Vec<IssueFieldDefinition> = serde_json::from_value(json!([
            { "uuid": "first", "name": "Estimate", "built_in": false, "type": { "value_type": "single_reference_object", "reference_object_type": "option" } },
            { "uuid": "second", "name": "Estimate", "built_in": false, "type": { "value_type": "integer" } },
            { "uuid": "empty", "name": "Empty", "built_in": false, "type": { "value_type": "text" } },
            { "uuid": "builtIn", "name": "Built in", "built_in": true, "type": { "value_type": "text" } },
            { "uuid": "linkedIssue", "name": "Linked issue", "built_in": false, "type": { "value_type": "multi_reference_object", "reference_object_type": "issue" } }
        ])).unwrap();
        let fields = parse_custom_fields(&item, &definitions);
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].value, "A");
        assert_eq!(fields[1].name, "Estimate");
    }
}
