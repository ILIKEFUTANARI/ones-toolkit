use super::{
    issue::parse_named_ref,
    value::{array_field, object_field, string_field},
};
use crate::models::{IssueRelatedSummary, RelatedCase, RelatedWikiPage};
use serde_json::Value;

pub fn parse_related_summary(value: Value) -> IssueRelatedSummary {
    let related_wiki_pages = array_field(&value, "relatedWikiPages")
        .into_iter()
        .map(|page| RelatedWikiPage {
            uuid: string_field(&page, "uuid"),
            title: string_field(&page, "title"),
            reference_type: string_field(&page, "referenceType")
                .or_else(|| string_field(&page, "ref_type")),
            sub_reference_type: string_field(&page, "subReferenceType")
                .or_else(|| string_field(&page, "sub_ref_type")),
            error_message: string_field(&page, "errorMessage"),
        })
        .collect();
    let related_cases = array_field(&value, "allRelatedCases")
        .into_iter()
        .map(|case| RelatedCase {
            uuid: string_field(&case, "uuid"),
            id: string_field(&case, "id"),
            name: string_field(&case, "name"),
            path: string_field(&case, "path"),
        })
        .collect();
    IssueRelatedSummary {
        can_edit: value.get("canEdit").and_then(Value::as_bool),
        sprint: object_field(&value, "sprint").map(parse_named_ref),
        related_wiki_pages_count: value.get("relatedWikiPagesCount").and_then(Value::as_u64),
        related_wiki_pages,
        related_cases,
    }
}
