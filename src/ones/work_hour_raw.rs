use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct GraphqlResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphqlError>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GraphqlError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhoursData {
    #[serde(default)]
    pub manhours: Vec<ManhourRecord>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourRecord {
    pub key: String,
    pub hours: i64,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub hour_type: String,
    pub owner: ManhourOwner,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourOwner {
    pub uuid: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourSummaryData {
    #[serde(default)]
    pub buckets: Vec<ManhourSummaryBucket>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourSummaryBucket {
    #[serde(rename = "actualHours", default)]
    pub actual_hours: i64,
    #[serde(rename = "estimatedHours", default)]
    pub estimated_hours: i64,
    #[serde(rename = "remainingHour", default)]
    pub remaining_hours: i64,
    pub owner: ManhourOwner,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourReportData {
    #[serde(default)]
    pub buckets: Vec<ManhourReportBucket>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourReportBucket {
    #[serde(rename = "columnField")]
    pub task: ManhourTask,
    #[serde(rename = "actualHours", default)]
    pub actual_hours: i64,
    #[serde(rename = "actualHoursSeries")]
    pub series: ManhourSeries,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourTask {
    pub uuid: String,
    #[serde(rename = "displayId")]
    pub display_id: Option<String>,
    pub name: Option<String>,
    pub project: Option<ManhourProject>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourProject {
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourSeries {
    #[serde(default)]
    pub times: Vec<String>,
    #[serde(default)]
    pub values: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourMutationData {
    #[serde(rename = "addManhour")]
    pub add: Option<ManhourKey>,
    #[serde(rename = "updateRemainingHour")]
    pub update_remaining: Option<ManhourKey>,
    #[serde(rename = "deleteManhour")]
    pub delete: Option<ManhourKey>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManhourKey {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TaskManhourPermissionData {
    pub task: TaskManhourPermission,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TaskManhourPermission {
    pub project: ManhourProject,
    #[serde(rename = "hasUpdateOwnAssessManHourPermission")]
    pub update_own_assess: bool,
    #[serde(rename = "hasUpdateAllAssessManHourPermission")]
    pub update_all_assess: bool,
    #[serde(rename = "hasUpdateOwnManHourPermission")]
    pub update_own_recorded: bool,
    #[serde(rename = "hasUpdateAllManHourPermission")]
    pub update_all_recorded: bool,
}
