use super::value::{display_value, format_timestamp, html_to_text, string_field};
use crate::models::IssueActivityEntry;
use serde_json::Value;

pub fn parse_issue_activity(value: Value) -> IssueActivityEntry {
    let ext = value.get("ext").cloned().unwrap_or(Value::Null);
    let field_name = string_field(&ext, "field_name");
    let old_value = activity_value(&ext, "old");
    let new_value = activity_value(&ext, "new");
    let text = ["content", "body", "comment", "text"]
        .into_iter()
        .find_map(|key| string_field(&value, key).or_else(|| string_field(&ext, key)))
        .map(|value| html_to_text(&value))
        .filter(|value| !value.is_empty());
    let action = string_field(&value, "action");
    let summary = match (&field_name, &action) {
        (Some(field), Some(action)) => Some(format!("{action} {field}")),
        (_, Some(action)) => Some(action.clone()),
        _ => string_field(&value, "object_name"),
    };

    IssueActivityEntry {
        entry_type: string_field(&value, "type"),
        actor: string_field(&value, "subject_name"),
        time: value.get("send_time").and_then(format_timestamp),
        summary,
        text,
        field_name,
        old_value,
        new_value,
    }
}

pub fn activity_cursor(value: &Value) -> Option<String> {
    string_field(value, "uuid")
}

fn activity_value(ext: &Value, prefix: &str) -> Option<String> {
    [
        format!("{prefix}_multi_option"),
        format!("{prefix}_option"),
        format!("{prefix}_value"),
    ]
    .into_iter()
    .find_map(|key| {
        ext.get(&key)
            .and_then(|value| display_value(value, &Value::Null))
    })
}
