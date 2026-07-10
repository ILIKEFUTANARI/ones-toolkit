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
        field::display_field_kind, field_catalog::CoreFields, identity::IdentityRegistry,
        raw::IssueFieldDefinition,
    },
};
use serde_json::Value;

pub fn parse_issue_detail(
    item: &Value,
    lookup: IssueLookupResult,
    definitions: &[IssueFieldDefinition],
    core: &CoreFields,
    related: Value,
    registry: &IdentityRegistry,
    redact: bool,
) -> IssueDetail {
    let mut identities = registry.scope(redact);
    let creator = object_field(item, &core.creator).and_then(|value| identities.person(&value));
    let assignee = object_field(item, &core.assignee).and_then(|value| identities.person(&value));
    let watchers = people(array_field(item, &core.watchers), &mut identities);
    let custom_fields = parse_custom_fields(item, definitions, &mut identities);
    let description = item
        .get(&core.description)
        .and_then(Value::as_str)
        .map(html_to_text)
        .filter(|value| !value.is_empty())
        .map(|value| identities.redact_text(value));

    IssueDetail {
        uuid: string_field(item, "uuid").unwrap_or(lookup.issue_uuid),
        display_id: string_field(item, &core.display_id).or(lookup.display_id),
        title: string_field(item, &core.title),
        issue_type: object_field(item, &core.issue_type).and_then(|value| ref_name(&value)),
        status: object_field(item, &core.status).and_then(|value| status_summary(&value)),
        project: object_field(item, &core.project).and_then(|value| ref_name(&value)),
        creator,
        assignee,
        watchers: non_empty(watchers),
        priority: object_field(item, &core.priority).and_then(|value| ref_name(&value)),
        sprint: object_field(item, &core.sprint).and_then(|value| ref_name(&value)),
        modules: non_empty(names(array_field(item, &core.modules))),
        due_date: string_field(item, &core.due_date),
        created_at: item.get(&core.created_at).and_then(format_timestamp),
        updated_at: item.get(&core.updated_at).and_then(format_timestamp),
        description,
        custom_fields: non_empty(custom_fields),
        related_wiki_pages: non_empty(parse_related_wiki_pages(&related)),
        related_cases: non_empty(parse_related_cases(&related)),
    }
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
