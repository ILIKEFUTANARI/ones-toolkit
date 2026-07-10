use super::value::{display_value, format_timestamp, html_to_text, string_field};
use crate::{models::IssueActivityEntry, ones::identity::IdentityRegistry};
use serde_json::Value;

pub fn parse_issue_activity(
    values: &[Value],
    registry: &IdentityRegistry,
    redact: bool,
) -> Vec<IssueActivityEntry> {
    let mut identities = registry.scope(redact);
    let actors = values
        .iter()
        .map(|value| {
            identities.person_parts(
                value.get("subject_id").and_then(Value::as_str),
                value.get("subject_name").and_then(Value::as_str),
            )
        })
        .collect::<Vec<_>>();

    values
        .iter()
        .zip(actors)
        .map(|(value, actor)| parse_entry(value, actor, &identities))
        .collect()
}

pub fn activity_cursor(value: &Value) -> Option<String> {
    string_field(value, "uuid")
}

fn parse_entry(
    value: &Value,
    actor: Option<String>,
    identities: &crate::ones::identity::IdentityScope<'_>,
) -> IssueActivityEntry {
    let ext = value.get("ext").cloned().unwrap_or(Value::Null);
    let field_name = string_field(&ext, "field_name");
    let action = string_field(value, "action");
    let summary = match (&field_name, &action) {
        (Some(field), Some(action)) => Some(format!("{action} {field}")),
        (_, Some(action)) => Some(action.clone()),
        _ => string_field(value, "object_name"),
    };
    let text = ["content", "body", "comment", "text"]
        .into_iter()
        .find_map(|key| string_field(value, key).or_else(|| string_field(&ext, key)))
        .map(|value| identities.redact_text(html_to_text(&value)))
        .filter(|value| !value.is_empty());

    IssueActivityEntry {
        entry_type: string_field(value, "type"),
        actor,
        time: value.get("send_time").and_then(format_timestamp),
        summary,
        text,
        field_name,
        old_value: activity_value(&ext, "old"),
        new_value: activity_value(&ext, "new"),
    }
}

fn activity_value(ext: &Value, prefix: &str) -> Option<String> {
    [
        format!("{prefix}_multi_option"),
        format!("{prefix}_option"),
        format!("{prefix}_value"),
    ]
    .into_iter()
    .find_map(|key| ext.get(&key).and_then(|value| display_value(value, None)))
}
