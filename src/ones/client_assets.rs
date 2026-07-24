use super::{
    assets_raw::{
        AttachmentDownloadResponse, AttachmentsResponse, BranchAddResponse,
        CodeAssociationResponse, CodeRepositoriesResponse,
    },
    client::OnesClient,
};
use crate::{error::AppResult, models::AuthContext};
use reqwest::Method;

impl OnesClient {
    pub async fn task_attachments(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        since: u64,
    ) -> AppResult<AttachmentsResponse> {
        self.request_json(
            Method::GET,
            auth,
            self.project_team_url(&format!(
                "task/{issue_uuid}/attachments?since={since}&field_uuids=all"
            )),
            None::<&()>,
        )
        .await
    }

    pub async fn attachment_download(
        &self,
        auth: &AuthContext,
        attachment_uuid: &str,
    ) -> AppResult<AttachmentDownloadResponse> {
        self.request_json(
            Method::GET,
            auth,
            self.project_team_url(&format!(
                "res/attachment/{attachment_uuid}?action=download&op="
            )),
            None::<&()>,
        )
        .await
    }

    pub async fn code_associations(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<CodeAssociationResponse> {
        let query = include_str!("graphql/code_associations.graphql");
        self.request_json(
            Method::POST,
            auth,
            self.project_team_url("items/graphql?t=Task"),
            Some(&serde_json::json!({
                "query": query,
                "variables": { "key": format!("task-{issue_uuid}") }
            })),
        )
        .await
    }

    pub async fn create_branch(
        &self,
        auth: &AuthContext,
        repo_uuid: &str,
        issue_uuid: &str,
        source_branch: &str,
        target_branch: &str,
    ) -> AppResult<BranchAddResponse> {
        self.request_json_once(
            Method::POST,
            auth,
            self.project_team_url(&format!("devops/scm/repo/{repo_uuid}/branch/add")),
            Some(&serde_json::json!({
                "task_uuid": issue_uuid,
                "source_branch": source_branch,
                "target_branch": target_branch,
            })),
        )
        .await
    }

    pub async fn code_repositories(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<CodeRepositoriesResponse> {
        let query = include_str!("graphql/code_repositories.graphql");
        self.request_json(
            Method::POST,
            auth,
            self.project_team_url("items/graphql?t=repo-data-key"),
            Some(&serde_json::json!({
                "query": query,
                "variables": {
                    "filter": {},
                    "orderBy": {"createTime": "DESC"},
                    "key": format!("task-{issue_uuid}")
                }
            })),
        )
        .await
    }
}
