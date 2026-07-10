use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    models::{
        AuthContext, IssueSummary, PageInfo, WorkbenchIssueGroup, WorkbenchViewDetail,
        WorkbenchViewSummary,
    },
    ones::{
        client::OnesClient,
        query::{
            build_group_count_query, build_issue_list_query, normalize_limit, parse_sort_fields,
        },
        raw::{PageInfoRaw, TreeNode, ViewDetailItem},
        transform::parse_issue_summary,
    },
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

mod issue;

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
