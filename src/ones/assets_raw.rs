use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AttachmentsResponse {
    #[serde(default)]
    pub attachments: Vec<AttachmentRecord>,
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub has_next: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentRecord {
    pub uuid: String,
    pub name: Option<String>,
    pub mime: Option<String>,
    pub size: Option<u64>,
    pub create_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct AttachmentDownloadResponse {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodeAssociationResponse {
    pub data: CodeAssociationData,
}

#[derive(Debug, Deserialize)]
pub struct CodeAssociationData {
    pub task: CodeAssociationTask,
}

#[derive(Debug, Deserialize)]
pub struct CodeAssociationTask {
    #[serde(default, rename = "devopsBranches")]
    pub branches: Vec<CodeBranchRecord>,
    #[serde(default, rename = "devopsTasksCommits")]
    pub commits: Vec<CodeCommitRecord>,
    #[serde(default, rename = "devopsPullRequests")]
    pub pull_requests: Vec<CodePullRequestRecord>,
}

#[derive(Debug, Deserialize)]
pub struct CodeRepositoriesResponse {
    pub data: CodeRepositoriesData,
}

#[derive(Debug, Deserialize)]
pub struct CodeRepositoriesData {
    pub task: CodeRepositoriesTask,
}

#[derive(Debug, Deserialize)]
pub struct CodeRepositoriesTask {
    #[serde(default, rename = "tStubTask")]
    pub repositories: Vec<CodeRepositoryRecord>,
}

#[derive(Debug, Deserialize)]
pub struct CodeRepositoryRecord {
    pub uuid: String,
    pub name: String,
    pub namespace: String,
    #[serde(rename = "scmType")]
    pub scm_type: String,
    pub auth: Option<CodeRepositoryAuth>,
}

#[derive(Debug, Deserialize)]
pub struct CodeRepositoryAuth {
    #[serde(rename = "isPublic")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CodeBranchRecord {
    pub uuid: String,
    pub name: String,
    pub url: Option<String>,
    pub repo: String,
    #[serde(rename = "scmType")]
    pub scm_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CodeCommitRecord {
    pub uuid: String,
    pub hash: Option<String>,
    pub message: Option<String>,
    pub branch: Option<String>,
    pub repo: String,
    pub scm: String,
    pub url: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodePullRequestRecord {
    pub uuid: String,
    pub key: Option<String>,
    pub number: Option<i64>,
    pub title: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "fromBranch")]
    pub from_branch: Option<String>,
    #[serde(rename = "toBranch")]
    pub to_branch: Option<String>,
    pub repo: String,
    #[serde(rename = "scmType")]
    pub scm_type: String,
    pub url: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BranchAddResponse {
    pub code: u16,
    pub errcode: String,
}
