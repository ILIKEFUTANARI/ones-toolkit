use crate::{
    error::{AppError, AppResult},
    models::{SortField, WorkbenchViewDetail},
    ones::{
        field_catalog::CoreFields,
        query::parse_sort_fields,
        raw::{TreeNode, ViewDetailItem, ViewHierarchyConfig},
    },
};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub(super) struct ResolvedView {
    pub(super) public: WorkbenchViewDetail,
    pub(super) onesql: String,
    pub(super) group_by: Option<String>,
    pub(super) sort: Vec<SortField>,
    hierarchy: Option<Value>,
}

pub(super) fn resolved_view_from_item(
    item: ViewDetailItem,
    core: &CoreFields,
) -> AppResult<ResolvedView> {
    let sort = parse_sort_fields(&item.sort);
    let onesql = item
        .condition
        .and_then(|condition| condition.onesql)
        .ok_or_else(|| AppError::invalid_upstream("view detail missing condition.onesql"))?;
    let hierarchy = item
        .hierarchy_config
        .as_ref()
        .filter(|config| config.enabled)
        .map(|config| build_hierarchy_payload(config, core))
        .transpose()?
        .flatten();
    Ok(ResolvedView {
        public: WorkbenchViewDetail {
            uuid: item.uuid.clone(),
            name: item.name.clone(),
            group_by: item.group_by.clone(),
            include_subtasks: item.include_subtasks,
            hierarchy_enabled: hierarchy.is_some(),
        },
        onesql,
        group_by: item.group_by,
        sort,
        hierarchy,
    })
}

pub(super) fn workitems_body(view: &ResolvedView, query: &str) -> Value {
    match &view.hierarchy {
        Some(hierarchy) => json!({ "query": query, "hierarchy": hierarchy }),
        None => json!({ "query": query }),
    }
}

fn build_hierarchy_payload(
    config: &ViewHierarchyConfig,
    core: &CoreFields,
) -> AppResult<Option<Value>> {
    let Some(tree) = config.tree_data.as_ref().map(tree_to_request) else {
        return Ok(None);
    };
    let upstream = config
        .upstream_field_uuid
        .clone()
        .or_else(|| core.parent.clone())
        .ok_or_else(|| AppError::invalid_upstream("parent hierarchy field is unavailable"))?;
    let downstream = config
        .downstream_field_uuid
        .clone()
        .or_else(|| core.children.clone())
        .ok_or_else(|| AppError::invalid_upstream("child hierarchy field is unavailable"))?;
    Ok(Some(json!({
        "lock_query": "",
        "perspective": config.perspective.unwrap_or(false),
        "flat": true,
        "path": { "upstream_field": upstream, "downstream_field": downstream },
        "config": { "field": core.issue_type, "tree": tree }
    })))
}

fn tree_to_request(node: &TreeNode) -> Value {
    let uuid = match node.uuid.as_str() {
        "ALLTYPES" => "__ALL__".to_string(),
        _ => node.uuid.clone(),
    };
    json!({
        "uuids": [uuid],
        "children": node.children.iter().map(tree_to_request).collect::<Vec<_>>(),
    })
}
