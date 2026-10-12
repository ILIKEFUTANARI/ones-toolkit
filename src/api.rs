use crate::attachments::IssueAttachmentPage;
use crate::code::{CreateBranchResult, IssueCodeLinks};
use crate::models::{
    IssueActivityPage, IssueDetail, IssueListItem, LegacyTaskInfo, PageInfo, WorkbenchIssueGroup,
    WorkbenchViewDetail, WorkbenchViewSummary,
};
use crate::ones::field_catalog::FieldDefinition;
use crate::work_hours::{IssueWorkHoursResponse, WorkHourMutationResponse, WorkHourReportResponse};
use crate::workflow::IssueTransitionsResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct ViewsResponse {
    pub views: Vec<WorkbenchViewSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct ViewResponse {
    pub view: WorkbenchViewDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct GroupsResponse {
    pub groups: Vec<WorkbenchIssueGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct WorkbenchIssuePage {
    pub issues: Vec<IssueListItem>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct IssueResponse {
    pub issue: IssueDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct FieldsResponse {
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "http", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct LegacyTaskInfoResponse {
    pub task_info: LegacyTaskInfo,
}

pub type ActivityResponse = IssueActivityPage;

pub type AttachmentsResponse = IssueAttachmentPage;

pub type CodeLinksResponse = IssueCodeLinks;

pub type CreateBranchResponse = CreateBranchResult;

pub type TransitionsResponse = IssueTransitionsResponse;
pub type WorkHoursResponse = IssueWorkHoursResponse;
pub type WorkHourReport = WorkHourReportResponse;
pub type WorkHourMutation = WorkHourMutationResponse;
