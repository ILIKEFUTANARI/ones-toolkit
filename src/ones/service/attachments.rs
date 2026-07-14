use super::OnesService;
use crate::{
    attachments::{IssueAttachment, IssueAttachmentPage},
    error::{AppError, AppResult},
    models::{AuthContext, PageInfo},
};

impl OnesService {
    pub async fn list_issue_attachments(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        cursor: Option<&str>,
    ) -> AppResult<IssueAttachmentPage> {
        let since = cursor
            .map(str::parse::<u64>)
            .transpose()
            .map_err(|_| AppError::invalid_input("attachment cursor must be an integer"))?
            .unwrap_or_default();
        let response = self
            .client
            .task_attachments(auth, issue_uuid, since)
            .await?;
        let next_cursor = if response.has_next {
            Some(
                response
                    .attachments
                    .iter()
                    .map(|item| item.create_time)
                    .max()
                    .ok_or_else(|| {
                        AppError::invalid_upstream("attachment page has_next without a next cursor")
                    })?
                    .to_string(),
            )
        } else {
            None
        };
        let mut attachments = Vec::with_capacity(response.attachments.len());

        for attachment in response.attachments {
            let details = self
                .client
                .attachment_download(auth, &attachment.uuid)
                .await?;
            attachments.push(IssueAttachment {
                uuid: attachment.uuid,
                name: attachment
                    .name
                    .unwrap_or_else(|| "unnamed attachment".to_string()),
                mime: attachment.mime,
                size: attachment.size,
                download_url: details.url,
            });
        }

        Ok(IssueAttachmentPage {
            attachments,
            page_info: PageInfo {
                has_next_page: response.has_next,
                next_cursor,
                total_count: Some(response.count),
            },
        })
    }
}
