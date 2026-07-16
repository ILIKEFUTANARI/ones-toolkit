use crate::attachments::IssueAttachmentPage;
use crate::code::{CreateBranchResult, IssueCodeLinks};
use crate::models::{
    IssueActivityPage, IssueDetail, IssueListItem, PageInfo, WorkbenchIssueGroup,
    WorkbenchViewDetail, WorkbenchViewSummary,
};
use crate::ones::field_catalog::FieldDefinition;
use crate::workflow::IssueTransitionsResponse;
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ViewsResponse {
    pub views: Vec<WorkbenchViewSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ViewResponse {
    pub view: WorkbenchViewDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct GroupsResponse {
    pub groups: Vec<WorkbenchIssueGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkbenchIssuePage {
    pub issues: Vec<IssueListItem>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueResponse {
    pub issue: IssueDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct FieldsResponse {
    pub fields: Vec<FieldDefinition>,
}

pub type ActivityResponse = IssueActivityPage;

pub type AttachmentsResponse = IssueAttachmentPage;

pub type CodeLinksResponse = IssueCodeLinks;

pub type CreateBranchResponse = CreateBranchResult;

pub type TransitionsResponse = IssueTransitionsResponse;
