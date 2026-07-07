use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    models::{
        AuthContext, IssueDetail, IssueLookupResult, IssueMessage, IssueRelatedSummary,
        IssueSummary, ListPinField, NamedRef, PageInfo, RelatedCase, RelatedWikiPage,
        WorkbenchIssueGroup, WorkbenchViewDetail, WorkbenchViewSummary,
    },
    ones::{
        client::OnesClient,
        query::{
            build_group_count_query, build_issue_detail_query, build_issue_list_query,
            normalize_limit, parse_sort_fields,
        },
        raw::{PageInfoRaw, TreeNode, ViewDetailItem},
    },
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct OnesService {
    client: Arc<OnesClient>,
}

impl OnesService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: Arc::new(OnesClient::new(config)),
        }
    }

    pub async fn list_workbench_views(
        &self,
        auth: &AuthContext,
    ) -> AppResult<Vec<WorkbenchViewSummary>> {
        let (list, config) = tokio::try_join!(
            self.client.user_filter_views_list(auth),
            self.client.user_filter_views_config(auth)
        )?;
        let visible: HashMap<_, _> = config
            .configs
            .into_iter()
            .map(|item| (item.view_uuid, item.is_show))
            .collect();

        Ok(list
            .views
            .into_iter()
            .map(|view| WorkbenchViewSummary {
                uuid: view.uuid.clone(),
                name: view.name,
                shared: view.shared,
                built_in: view.built_in,
                is_show: visible.get(&view.uuid).copied().unwrap_or(false),
            })
            .collect())
    }

    pub async fn get_workbench_view(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
    ) -> AppResult<WorkbenchViewDetail> {
        Ok(self.resolve_view(auth, view_uuid).await?.public)
    }

    pub async fn list_workbench_issue_groups(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
    ) -> AppResult<Vec<WorkbenchIssueGroup>> {
        let view = self.resolve_view(auth, view_uuid).await?;

        if let Some(group_by) = view.group_by.as_deref() {
            let query = build_group_count_query(&view.onesql, group_by, &view.sort);
            let body = self.workitems_body(&view, &query);
            let response = self.client.workitems_onesql(auth, &body).await?;
            return Ok(parse_group_response(&response.data, group_by));
        }

        let issues = self
            .list_workbench_issues(auth, view_uuid, None, None, Some(1))
            .await?;
        Ok(vec![WorkbenchIssueGroup {
            value: None,
            label: "all".to_string(),
            total_count: issues
                .page_info
                .total_count
                .unwrap_or(issues.issues.len() as u64),
            hierarchy_count: None,
        }])
    }

    pub async fn list_workbench_issues(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
        group_value: Option<&str>,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> AppResult<WorkbenchIssuePage> {
        let view = self.resolve_view(auth, view_uuid).await?;
        let limit = normalize_limit(limit);
        let query = build_issue_list_query(
            &view.onesql,
            view.group_by.as_deref(),
            group_value,
            cursor,
            limit,
            &view.sort,
        )?;
        let body = self.workitems_body(&view, &query);
        let response = self.client.workitems_onesql(auth, &body).await?;

        Ok(WorkbenchIssuePage {
            issues: response
                .data
                .into_iter()
                .filter_map(|entry| entry.get("item").cloned())
                .map(parse_issue_summary)
                .collect(),
            page_info: response.page_info.map(to_page_info).unwrap_or_default(),
        })
    }

    pub async fn get_issue_by_display_id(
        &self,
        auth: &AuthContext,
        display_id: &str,
    ) -> AppResult<IssueDetail> {
        let lookup = self.resolve_display_id(auth, display_id).await?;
        self.get_issue_detail_with_lookup(auth, lookup).await
    }

    pub async fn get_issue_detail(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<IssueDetail> {
        let lookup = IssueLookupResult {
            issue_uuid: issue_uuid.to_string(),
            display_id: None,
            identifier: None,
            path: Some(issue_uuid.to_string()),
        };
        self.get_issue_detail_with_lookup(auth, lookup).await
    }

    async fn get_issue_detail_with_lookup(
        &self,
        auth: &AuthContext,
        lookup: IssueLookupResult,
    ) -> AppResult<IssueDetail> {
        let detail_query = build_issue_detail_query(&lookup.issue_uuid);
        let body = json!({ "query": detail_query });

        let (detail, detail_form, fields, messages, related) = tokio::try_join!(
            self.client.workitems_onesql(auth, &body),
            self.client.issue_detail_form(auth, &lookup.issue_uuid),
            self.client.issue_form_fields(auth, &lookup.issue_uuid),
            self.client.task_messages(auth, &lookup.issue_uuid),
            self.client.quick_action_task_info(auth, &lookup.issue_uuid),
        )?;

        let detail_item = detail
            .data
            .into_iter()
            .find_map(|entry| entry.get("item").cloned())
            .ok_or_else(|| AppError::NotFound(format!("issue {}", lookup.issue_uuid)))?;
        let mut summary = parse_issue_summary(detail_item.clone());
        if summary.display_id.is_none() {
            summary.display_id = lookup.display_id.clone();
        }

        Ok(IssueDetail {
            lookup,
            summary,
            detail_item,
            detail_form,
            form_fields: fields.fields,
            messages: messages
                .messages
                .into_iter()
                .map(parse_issue_message)
                .collect(),
            related: Some(parse_related_summary(related.data.task)),
        })
    }

    async fn resolve_display_id(
        &self,
        auth: &AuthContext,
        display_id: &str,
    ) -> AppResult<IssueLookupResult> {
        let response = self.client.resolve_display_id(auth, display_id).await?;
        let issue_uuid = response
            .task_uuid
            .ok_or_else(|| AppError::NotFound(format!("display id {display_id}")))?;
        Ok(IssueLookupResult {
            issue_uuid,
            display_id: response.display_id.or(Some(display_id.to_string())),
            identifier: response.identifier,
            path: response.path.or(response.display_id_path),
        })
    }

    async fn resolve_view(&self, auth: &AuthContext, view_uuid: &str) -> AppResult<ResolvedView> {
        let detail = self
            .client
            .user_filter_views_detail(auth, view_uuid)
            .await?;
        let item = detail
            .views
            .into_iter()
            .find(|view| view.uuid == view_uuid)
            .ok_or_else(|| AppError::NotFound(format!("view {view_uuid}")))?;
        resolved_view_from_item(item)
    }

    fn workitems_body(&self, view: &ResolvedView, query: &str) -> Value {
        match &view.hierarchy {
            Some(hierarchy) => json!({ "query": query, "hierarchy": hierarchy }),
            None => json!({ "query": query }),
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedView {
    public: WorkbenchViewDetail,
    onesql: String,
    group_by: Option<String>,
    sort: Vec<crate::models::SortField>,
    hierarchy: Option<Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkbenchIssuePage {
    pub issues: Vec<IssueSummary>,
    pub page_info: PageInfo,
}

fn resolved_view_from_item(item: ViewDetailItem) -> AppResult<ResolvedView> {
    let sort = parse_sort_fields(&item.sort);
    let onesql = item
        .condition
        .and_then(|condition| condition.onesql)
        .ok_or_else(|| AppError::invalid_upstream("view detail missing condition.onesql"))?;
    let hierarchy = item
        .hierarchy_config
        .as_ref()
        .filter(|config| config.enabled)
        .and_then(build_hierarchy_payload);

    Ok(ResolvedView {
        public: WorkbenchViewDetail {
            uuid: item.uuid.clone(),
            name: item.name.clone(),
            group_by: item.group_by.clone(),
            layout: item.layout.clone(),
            display_type: item.display_type.clone(),
            shared: item.shared,
            built_in: item.built_in,
            include_subtasks: item.include_subtasks,
            is_fold_all_groups: item.is_fold_all_groups,
            sort: sort.clone(),
            hierarchy_enabled: hierarchy.is_some(),
        },
        onesql,
        group_by: item.group_by,
        sort,
        hierarchy,
    })
}

fn build_hierarchy_payload(config: &crate::ones::raw::ViewHierarchyConfig) -> Option<Value> {
    let tree = config.tree_data.as_ref().map(tree_to_request)?;
    Some(json!({
        "lock_query": "",
        "perspective": config.perspective.unwrap_or(false),
        "flat": true,
        "path": {
            "upstream_field": config.upstream_field_uuid.clone().unwrap_or_else(|| "field014".to_string()),
            "downstream_field": config.downstream_field_uuid.clone().unwrap_or_else(|| "field114".to_string()),
        },
        "config": {
            "field": "field007",
            "tree": tree,
        }
    }))
}

fn tree_to_request(node: &TreeNode) -> Value {
    let uuid = if node.uuid == "ALLTYPES" {
        "__ALL__".to_string()
    } else {
        node.uuid.clone()
    };
    json!({
        "uuids": [uuid],
        "children": node.children.iter().map(tree_to_request).collect::<Vec<_>>(),
    })
}

fn parse_group_response(data: &[Value], group_by: &str) -> Vec<WorkbenchIssueGroup> {
    data.iter()
        .find_map(|entry| entry.get("group_aggregate"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| WorkbenchIssueGroup {
                    value: item
                        .get(group_by)
                        .and_then(|value| value.get("value"))
                        .and_then(Value::as_str)
                        .map(ToString::to_string),
                    label: item
                        .get(group_by)
                        .and_then(|value| value.get("label"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                    total_count: item
                        .get("total_count")
                        .and_then(Value::as_u64)
                        .unwrap_or_default(),
                    hierarchy_count: item.get("hierarchy_count").and_then(Value::as_u64),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_issue_summary(item: Value) -> IssueSummary {
    let assignees = array_field(&item, "field008")
        .into_iter()
        .map(parse_named_ref)
        .collect();
    let raw = item.clone();
    IssueSummary {
        uuid: string_field(&item, "uuid").unwrap_or_default(),
        path: string_field(&item, "path"),
        display_id: string_field(&item, "display_id_path")
            .or_else(|| string_field(&item, "field903")),
        title: string_field(&item, "field001"),
        issue_type: object_field(&item, "field007").map(parse_named_ref),
        status: object_field(&item, "field005").map(parse_named_ref),
        project: object_field(&item, "field006").map(parse_named_ref),
        assignees,
        due_date: string_field(&item, "field013"),
        has_manhour_tab: item
            .get("has_manhour_tab")
            .and_then(Value::as_i64)
            .map(|value| value != 0),
        downstream_task_count: item.get("downstream_task_count").and_then(Value::as_u64),
        downstream_task_done_count: item
            .get("downstream_task_done_count")
            .and_then(Value::as_u64),
        list_pin_fields: array_field(&item, "list_pin_field")
            .into_iter()
            .map(parse_list_pin_field)
            .collect(),
        raw,
    }
}

fn parse_named_ref(value: Value) -> NamedRef {
    NamedRef {
        uuid: string_field(&value, "uuid"),
        name: string_field(&value, "name"),
        icon: value.get("icon").and_then(Value::as_i64),
        icon_uuid: string_field(&value, "icon_uuid"),
        detail_type: value.get("detail_type").and_then(Value::as_i64),
        category: string_field(&value, "category")
            .or_else(|| string_field(&value, "status_category")),
    }
}

fn parse_list_pin_field(value: Value) -> ListPinField {
    ListPinField {
        field_uuid: string_field(&value, "field_uuid").unwrap_or_default(),
        name: string_field(&value, "name").unwrap_or_default(),
        value: string_field(&value, "value").unwrap_or_default(),
        bg_color: string_field(&value, "bg_color"),
        color: string_field(&value, "color"),
    }
}

fn parse_issue_message(value: Value) -> IssueMessage {
    IssueMessage {
        uuid: string_field(&value, "uuid"),
        message_type: string_field(&value, "type"),
        subject_name: string_field(&value, "subject_name"),
        action: string_field(&value, "action"),
        object_type: string_field(&value, "object_type"),
        object_id: string_field(&value, "object_id"),
        object_name: string_field(&value, "object_name"),
        send_time: value.get("send_time").and_then(Value::as_i64),
        raw: value,
    }
}

fn parse_related_summary(value: Value) -> IssueRelatedSummary {
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
        raw: value,
    }
}

fn to_page_info(raw: PageInfoRaw) -> PageInfo {
    PageInfo {
        has_next_page: raw.has_next_page,
        start_cursor: raw.start_cursor,
        end_cursor: raw.end_cursor,
        page_count: raw.page_count,
        total_count: raw.total_count,
        start_pos: raw.start_pos,
        end_pos: raw.end_pos,
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn object_field(value: &Value, key: &str) -> Option<Value> {
    value.get(key).cloned().filter(|item| item.is_object())
}

fn array_field(value: &Value, key: &str) -> Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_lookup_and_summary() {
        let item = json!({
            "uuid": "issue-demo-001",
            "display_id_path": "DEMO-1234",
            "field001": "Sample issue for integration testing",
            "field005": { "name": "Done", "category": "done", "uuid": "status-demo-done" },
            "field006": { "uuid": "proj-demo-001", "name": "Demo Project" },
            "field007": { "uuid": "type-demo-bug", "name": "Bug", "icon": 9, "detail_type": 0 },
            "field008": [{ "uuid": "user-demo-001", "name": "Alex Example" }]
        });
        let summary = parse_issue_summary(item);
        assert_eq!(summary.display_id.as_deref(), Some("DEMO-1234"));
        assert_eq!(
            summary.project.and_then(|p| p.name).as_deref(),
            Some("Demo Project")
        );
    }
}
