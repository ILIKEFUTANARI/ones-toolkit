use crate::ones::raw::IssueFieldDefinition;

const CORE_FIELD_UUIDS: &[&str] = &[
    "uuid",
    "path",
    "v$issue_path",
    "field001",
    "field003",
    "field004",
    "field005",
    "field006",
    "field007",
    "field008",
    "field009",
    "field010",
    "field011",
    "field012",
    "field013",
    "field016",
    "field054",
    "field903",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayFieldKind {
    Scalar,
    Reference,
}

pub fn display_field_kind(definition: &IssueFieldDefinition) -> Option<DisplayFieldKind> {
    if definition.built_in
        || is_core_field_uuid(&definition.uuid)
        || !is_onesql_identifier(&definition.uuid)
    {
        return None;
    }

    let field_type = definition.field_type.as_ref()?;
    match field_type.value_type.as_deref()? {
        "text" | "multi_line_text" | "richtext" | "integer" | "float" | "date" | "datetime"
        | "duration_hour" | "duration_second" | "boolean" => Some(DisplayFieldKind::Scalar),
        "single_reference_object" | "multi_reference_object"
            if matches!(
                field_type.reference_object_type.as_deref(),
                Some("user" | "option")
            ) =>
        {
            Some(DisplayFieldKind::Reference)
        }
        _ => None,
    }
}

fn is_core_field_uuid(field_uuid: &str) -> bool {
    CORE_FIELD_UUIDS.contains(&field_uuid)
}

fn is_onesql_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}
