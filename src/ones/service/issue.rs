use super::OnesService;
use crate::{
    error::{AppError, AppResult},
    models::{ActivityPageInfo, AuthContext, IssueActivityPage, IssueDetail, IssueLookupResult},
    ones::{
        query::{build_issue_detail_query, normalize_limit},
        transform::{
            activity_cursor, parse_core_fields, parse_custom_fields, parse_issue_activity,
            parse_issue_summary, parse_related_summary,
        },
    },
};
use serde_json::json;

impl OnesService {
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
        let detail_request = async {
            let fields = self
                .client
                .issue_form_fields(auth, &lookup.issue_uuid)
                .await?;
            let detail_query = build_issue_detail_query(&lookup.issue_uuid, &fields.fields);
            let body = json!({ "query": detail_query });
            let detail = self.client.workitems_onesql(auth, &body).await?;
            Ok::<_, AppError>((detail, fields))
        };
        let ((detail, fields), related) = tokio::try_join!(
            detail_request,
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
            core_fields: parse_core_fields(&detail_item),
            custom_fields: parse_custom_fields(&detail_item, &fields.fields),
            related: parse_related_summary(related.data.task),
        })
    }

    pub async fn list_issue_activity(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> AppResult<IssueActivityPage> {
        let messages = self.client.task_messages(auth, issue_uuid).await?.messages;
        let start = match cursor {
            Some(cursor) => messages
                .iter()
                .position(|message| activity_cursor(message).as_deref() == Some(cursor))
                .map(|position| position + 1)
                .ok_or_else(|| AppError::invalid_input("activity cursor was not found"))?,
            None => 0,
        };
        let limit = normalize_limit(limit) as usize;
        let end = (start + limit).min(messages.len());
        let activity = messages[start..end]
            .iter()
            .cloned()
            .map(parse_issue_activity)
            .collect();

        Ok(IssueActivityPage {
            activity,
            page_info: ActivityPageInfo {
                has_next_page: end < messages.len(),
                next_cursor: (end < messages.len())
                    .then(|| activity_cursor(&messages[end - 1]))
                    .flatten(),
            },
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
}
