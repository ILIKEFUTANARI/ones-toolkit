use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct CreateBranchInput {
    pub repo_uuid: String,
    pub source_branch: String,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct CreateBranchResult {
    pub issue_uuid: String,
    pub source_branch: String,
    pub target_branch: String,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct IssueCodeLinks {
    pub repositories: Vec<CodeRepository>,
    pub branches: Vec<CodeBranch>,
    pub commits: Vec<CodeCommit>,
    pub pull_requests: Vec<CodePullRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct CodeRepository {
    pub uuid: String,
    pub name: String,
    pub namespace: String,
    pub scm_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct CodeBranch {
    pub uuid: String,
    pub name: String,
    pub repo: String,
    pub scm_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct CodeCommit {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub repo: String,
    pub scm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct CodePullRequest {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_branch: Option<String>,
    pub repo: String,
    pub scm_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}
