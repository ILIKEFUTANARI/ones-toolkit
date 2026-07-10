use super::value::{array_field, string_field};
use crate::models::{RelatedCase, RelatedWikiPage};
use serde_json::Value;

pub fn parse_related_wiki_pages(value: &Value) -> Vec<RelatedWikiPage> {
    array_field(value, "relatedWikiPages")
        .iter()
        .map(|page| RelatedWikiPage {
            title: string_field(page, "title"),
            error_message: string_field(page, "errorMessage"),
        })
        .collect()
}

pub fn parse_related_cases(value: &Value) -> Vec<RelatedCase> {
    array_field(value, "allRelatedCases")
        .iter()
        .map(|case| RelatedCase {
            name: string_field(case, "name"),
        })
        .collect()
}
