use super::{
    issue::{ref_name, status_summary},
    related::{parse_related_cases, parse_related_wiki_pages},
    value::{
        array_field, display_value, format_timestamp, html_to_text, object_field, string_field,
    },
};
use crate::{
    models::{DisplayField, IssueDetail, IssueLookupResult},
    ones::{
        field::{definition_by_type, display_field_kind},
        identity::IdentityRegistry,
        raw::IssueFieldDefinition,
    },
};
use serde_json::Value;

pub fn parse_issue_detail(
    item: &Value,
    lookup: IssueLookupResult,
    definitions: &[IssueFieldDefinition],
    related: Value,
    registry: &IdentityRegistry,
    redact: bool,
) -> IssueDetail {
    let mut identities = registry.scope(redact);
    let creator =
        object_by_type(item, definitions, "creator").and_then(|value| identities.person(&value));
    let assignee =
        object_by_type(item, definitions, "assign").and_then(|value| identities.person(&value));
    let watchers = people(array_by_type(item, definitions, "watcher"), &mut identities);
    let custom_fields = parse_custom_fields(item, definitions, &mut identities);
    let description = value_by_type(item, definitions, "rich_desc")
        .and_then(Value::as_str)
        .map(html_to_text)
        .filter(|value| !value.is_empty())
        .map(|value| identities.redact_text(value));

    IssueDetail {
        uuid: string_field(item, "uuid").unwrap_or(lookup.issue_uuid),
        display_id: string_by_type(item, definitions, "display_id").or(lookup.display_id),
        title: string_by_type(item, definitions, "name"),
        issue_type: object_by_type(item, definitions, "issue_type")
            .and_then(|value| ref_name(&value)),
        status: object_by_type(item, definitions, "status")
            .and_then(|value| status_summary(&value)),
        project: object_by_type(item, definitions, "project").and_then(|value| ref_name(&value)),
        creator,
        assignee,
        watchers: non_empty(watchers),
        priority: object_by_type(item, definitions, "priority").and_then(|value| ref_name(&value)),
        sprint: object_by_type(item, definitions, "sprint").and_then(|value| ref_name(&value)),
        modules: non_empty(names(array_by_type(
            item,
            definitions,
            "multi_project_module",
        ))),
        due_date: string_by_type(item, definitions, "date"),
        created_at: value_by_type(item, definitions, "create_time").and_then(format_timestamp),
        updated_at: value_by_type(item, definitions, "update_time").and_then(format_timestamp),
        description,
        custom_fields: non_empty(custom_fields),
        related_wiki_pages: non_empty(parse_related_wiki_pages(&related)),
        related_cases: non_empty(parse_related_cases(&related)),
    }
}

fn value_by_type<'a>(
    item: &'a Value,
    definitions: &[IssueFieldDefinition],
    type_uuid: &str,
) -> Option<&'a Value> {
    let definition = definition_by_type(definitions, type_uuid)?;
    item.get(&definition.uuid)
}

fn string_by_type(
    item: &Value,
    definitions: &[IssueFieldDefinition],
    type_uuid: &str,
) -> Option<String> {
    let definition = definition_by_type(definitions, type_uuid)?;
    string_field(item, &definition.uuid)
}

fn object_by_type(
    item: &Value,
    definitions: &[IssueFieldDefinition],
    type_uuid: &str,
) -> Option<Value> {
    let definition = definition_by_type(definitions, type_uuid)?;
    object_field(item, &definition.uuid)
}

fn array_by_type(
    item: &Value,
    definitions: &[IssueFieldDefinition],
    type_uuid: &str,
) -> Vec<Value> {
    definition_by_type(definitions, type_uuid)
        .map(|definition| array_field(item, &definition.uuid))
        .unwrap_or_default()
}

fn parse_custom_fields(
    item: &Value,
    definitions: &[IssueFieldDefinition],
    identities: &mut crate::ones::identity::IdentityScope<'_>,
) -> Vec<DisplayField> {
    definitions
        .iter()
        .filter_map(|definition| {
            display_field_kind(definition)?;
            let field_type = definition.field_type.as_ref()?;
            let raw = item.get(&definition.uuid)?;
            let value = if field_type.reference_object_type.as_deref() == Some("user") {
                user_value(raw, identities)?
            } else {
                display_value(raw, field_type.value_type.as_deref())?
            };
            Some(DisplayField {
                name: definition
                    .name
                    .clone()
                    .unwrap_or_else(|| "unnamed field".to_string()),
                value,
            })
        })
        .collect()
}

fn user_value(
    value: &Value,
    identities: &mut crate::ones::identity::IdentityScope<'_>,
) -> Option<String> {
    match value {
        Value::Object(_) => identities.person(value),
        Value::Array(values) => {
            let values = values
                .iter()
                .filter_map(|value| identities.person(value))
                .collect::<Vec<_>>();
            (!values.is_empty()).then(|| values.join(", "))
        }
        Value::String(value) => identities.person_parts(None, Some(value)),
        _ => None,
    }
}

fn people(
    values: Vec<Value>,
    identities: &mut crate::ones::identity::IdentityScope<'_>,
) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| identities.person(value))
        .collect()
}

fn names(values: Vec<Value>) -> Vec<String> {
    values.iter().filter_map(ref_name).collect()
}

fn non_empty<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
}
