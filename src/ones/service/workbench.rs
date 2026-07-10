use super::OnesService;
use crate::{
    api::WorkbenchIssuePage,
    error::{AppError, AppResult},
    models::{
        AuthContext, PageInfo, WorkbenchIssueGroup, WorkbenchViewDetail, WorkbenchViewSummary,
    },
    ones::{
        field_catalog::CoreFields,
        query::{build_group_count_query, build_issue_list_query, normalize_limit},
        raw::PageInfoRaw,
        transform::parse_issue_list_item,
    },
};
use serde_json::Value;

mod view;
use view::{ResolvedView, resolved_view_from_item, workitems_body};

impl OnesService {
    pub async fn list_workbench_views(
        &self,
        auth: &AuthContext,
    ) -> AppResult<Vec<WorkbenchViewSummary>> {
        Ok(self
            .client
            .user_filter_views_list(auth)
            .await?
            .views
            .into_iter()
            .map(|view| WorkbenchViewSummary {
                uuid: view.uuid,
                name: view.name,
            })
            .collect())
    }

    pub async fn get_workbench_view(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
    ) -> AppResult<WorkbenchViewDetail> {
        let catalog = self.team_field_catalog(auth).await?;
        Ok(self
            .resolve_view(auth, view_uuid, &catalog.core)
            .await?
            .public)
    }

    pub async fn list_workbench_issue_groups(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
    ) -> AppResult<Vec<WorkbenchIssueGroup>> {
        let catalog = self.team_field_catalog(auth).await?;
        let view = self.resolve_view(auth, view_uuid, &catalog.core).await?;
        if let Some(group_by) = view.group_by.as_deref() {
            catalog.validate_group_field(group_by)?;
            let query = build_group_count_query(&view.onesql, group_by, &view.sort, &catalog.core);
            let response = self
                .client
                .workitems_onesql(auth, &workitems_body(&view, &query))
                .await?;
            return Ok(parse_group_response(&response.data, group_by));
        }
        let issues = self
            .list_workbench_issues(auth, view_uuid, None, None, Some(1), true)
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
        redact: bool,
    ) -> AppResult<WorkbenchIssuePage> {
        let catalog = self.team_field_catalog(auth).await?;
        let view = self.resolve_view(auth, view_uuid, &catalog.core).await?;
        let query = build_issue_list_query(
            &view.onesql,
            view.group_by.as_deref(),
            group_value,
            cursor,
            normalize_limit(limit),
            &view.sort,
            &catalog.core,
        )?;
        let response = self
            .client
            .workitems_onesql(auth, &workitems_body(&view, &query))
            .await?;
        let issues = response
            .data
            .into_iter()
            .filter_map(|entry| entry.get("item").cloned())
            .map(|item| parse_issue_list_item(item, &catalog.core, &self.identities, redact))
            .collect();
        Ok(WorkbenchIssuePage {
            issues,
            page_info: response.page_info.map(to_page_info).unwrap_or_default(),
        })
    }

    async fn resolve_view(
        &self,
        auth: &AuthContext,
        view_uuid: &str,
        core: &CoreFields,
    ) -> AppResult<ResolvedView> {
        let detail = self
            .client
            .user_filter_views_detail(auth, view_uuid)
            .await?;
        let item = detail
            .views
            .into_iter()
            .find(|view| view.uuid == view_uuid)
            .ok_or_else(|| AppError::NotFound(format!("view {view_uuid}")))?;
        resolved_view_from_item(item, core)
    }
}

fn parse_group_response(data: &[Value], group_by: &str) -> Vec<WorkbenchIssueGroup> {
    data.iter()
        .find_map(|entry| entry.get("group_aggregate"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| WorkbenchIssueGroup {
                    value: nested_string(item, group_by, "value"),
                    label: nested_string(item, group_by, "label")
                        .unwrap_or_else(|| "unknown".to_string()),
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

fn nested_string(value: &Value, field: &str, key: &str) -> Option<String> {
    value
        .get(field)?
        .get(key)?
        .as_str()
        .map(ToString::to_string)
}

fn to_page_info(raw: PageInfoRaw) -> PageInfo {
    PageInfo {
        has_next_page: raw.has_next_page,
        next_cursor: raw.has_next_page.then_some(raw.end_cursor).flatten(),
        total_count: raw.total_count,
    }
}
