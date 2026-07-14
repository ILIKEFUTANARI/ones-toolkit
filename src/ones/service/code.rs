use super::OnesService;
use crate::{
    code::{
        CodeBranch, CodeCommit, CodePullRequest, CodeRepository, CreateBranchInput,
        CreateBranchResult, IssueCodeLinks,
    },
    error::{AppError, AppResult},
    models::AuthContext,
};

impl OnesService {
    pub async fn list_issue_code_links(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        redact: bool,
    ) -> AppResult<IssueCodeLinks> {
        let (response, repositories_response) = tokio::try_join!(
            self.client.code_associations(auth, issue_uuid),
            self.client.code_repositories(auth, issue_uuid),
        )?;
        let mut identities = self.identities.scope(redact);
        let task = response.data.task;

        let repositories = repositories_response
            .data
            .task
            .repositories
            .into_iter()
            .map(|repository| CodeRepository {
                uuid: repository.uuid,
                name: repository.name,
                namespace: repository.namespace,
                scm_type: repository.scm_type,
                is_public: repository.auth.and_then(|auth| auth.is_public),
            })
            .collect();

        let branches = task
            .branches
            .into_iter()
            .map(|branch| CodeBranch {
                uuid: branch.uuid,
                name: branch.name,
                repo: branch.repo,
                scm_type: branch.scm_type,
                url: branch.url,
            })
            .collect();
        let commits = task
            .commits
            .into_iter()
            .map(|commit| CodeCommit {
                uuid: commit.uuid,
                hash: commit.hash,
                message: commit.message,
                branch: commit.branch,
                repo: commit.repo,
                scm: commit.scm,
                url: commit.url,
                author: redact_name(&mut identities, commit.author),
            })
            .collect();
        let pull_requests = task
            .pull_requests
            .into_iter()
            .map(|request| CodePullRequest {
                uuid: request.uuid,
                key: request.key,
                number: request.number,
                title: request.title,
                state: request.state,
                from_branch: request.from_branch,
                to_branch: request.to_branch,
                repo: request.repo,
                scm_type: request.scm_type,
                url: request.url,
                author: redact_name(&mut identities, request.author),
            })
            .collect();

        Ok(IssueCodeLinks {
            repositories,
            branches,
            commits,
            pull_requests,
        })
    }

    pub async fn create_issue_branch(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        input: &CreateBranchInput,
    ) -> AppResult<CreateBranchResult> {
        validate_branch_input("issue_uuid", issue_uuid)?;
        validate_branch_input("repo_uuid", &input.repo_uuid)?;
        validate_branch_input("source_branch", &input.source_branch)?;
        validate_branch_input("target_branch", &input.target_branch)?;
        if input.source_branch == input.target_branch {
            return Err(AppError::invalid_input(
                "source_branch and target_branch must differ",
            ));
        }

        let response = self
            .client
            .create_branch(
                auth,
                &input.repo_uuid,
                issue_uuid,
                &input.source_branch,
                &input.target_branch,
            )
            .await?;
        if response.code != 200 || response.errcode != "OK" {
            return Err(AppError::invalid_upstream(format!(
                "ONES branch creation failed: code={}, errcode={}",
                response.code, response.errcode
            )));
        }

        Ok(CreateBranchResult {
            issue_uuid: issue_uuid.to_string(),
            source_branch: input.source_branch.clone(),
            target_branch: input.target_branch.clone(),
            created: true,
        })
    }
}

fn redact_name(
    identities: &mut crate::ones::identity::IdentityScope<'_>,
    name: Option<String>,
) -> Option<String> {
    name.and_then(|name| identities.person_parts(None, Some(&name)))
}

fn validate_branch_input(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(AppError::invalid_input(format!(
            "{field} must be non-empty and contain no control characters"
        )));
    }
    Ok(())
}
