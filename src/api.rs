use crate::models::{
    IssueActivityPage, IssueDetail, IssueListItem, PageInfo, WorkbenchIssueGroup,
    WorkbenchViewDetail, WorkbenchViewSummary,
};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ViewsResponse {
    pub views: Vec<WorkbenchViewSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ViewResponse {
    pub view: WorkbenchViewDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GroupsResponse {
    pub groups: Vec<WorkbenchIssueGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkbenchIssuePage {
    pub issues: Vec<IssueListItem>,
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IssueResponse {
    pub issue: IssueDetail,
}

pub type ActivityResponse = IssueActivityPage;
