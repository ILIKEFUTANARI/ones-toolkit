use crate::ones::raw::IssueFieldDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayFieldKind {
    Scalar,
    Reference,
}

pub fn display_field_kind(definition: &IssueFieldDefinition) -> Option<DisplayFieldKind> {
    if definition.built_in || !is_onesql_identifier(&definition.uuid) {
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

pub fn definition_by_type<'a>(
    definitions: &'a [IssueFieldDefinition],
    type_uuid: &str,
) -> Option<&'a IssueFieldDefinition> {
    definitions.iter().find(|definition| {
        definition
            .field_type
            .as_ref()
            .and_then(|field_type| field_type.uuid.as_deref())
            == Some(type_uuid)
    })
}

pub fn is_onesql_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}
