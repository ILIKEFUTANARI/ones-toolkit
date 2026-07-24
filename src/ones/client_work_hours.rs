use super::{
    client::OnesClient,
    work_hour_raw::{
        GraphqlResponse, ManhourMutationData, ManhourReportData, ManhourSummaryData, ManhoursData,
        TaskManhourPermissionData,
    },
};
use crate::{
    error::{AppError, AppResult},
    models::AuthContext,
};
use reqwest::Method;
use serde_json::{Value, json};

const ISSUE_MANHOURS_QUERY: &str = r#"
    query FetchUserManhours {
      manhours(
        filter: $manhourFilter
        orderBy: $manhoursOrderBy
      ) {
        key
        hours
        startTime(unit: ONESDATE)
        description
        type
        owner {
          uuid
          name
        }
      }
    }
"#;

const ISSUE_SUMMARY_QUERY: &str = r#"
    query FetchUserManhoursInfo {
      buckets (
        groupBy: $groupBy
        orderBy: $orderBy
      ) {
        manhours ( filter: $manhourFilter ) { key }
        owner {
          uuid
          name
        }
        actualHours(sum: "manhours.recordedHour")
        estimatedHours(sum: "manhours.estimatedHour")
        remainingHour
      }
    }
"#;

const REPORT_QUERY: &str = r#"
    query QUERY_MANHOURS {
      buckets(groupBy: $groupBy, orderBy: $orderBy, filter: $filter) {
        columnField: aggregateTask(source: $columnSource) {
          uuid
          name
          displayId
          project { uuid }
        }
        key
        actualHours(sum: $actualHoursSum)
        actualHoursSeries(timeSeries: $timeSeries) {
          times
          values
        }
      }
    }
"#;

const ADD_MANHOUR_MUTATION: &str = r#"
    mutation AddManhour {
      addManhour(
        mode: $mode
        owner: $owner
        task: $task
        type: $type
        start_time: $start_time
        hours: $hours
        description: $description
        customData: $customData
      ) { key }
    }
"#;

const UPDATE_REMAINING_MUTATION: &str = r#"
    mutation UpdateRemainingHour {
      updateRemainingHour(mode: $mode, key: $key, hours: $hours) { key }
    }
"#;

const DELETE_MANHOUR_MUTATION: &str = r#"
    mutation DeleteManhour {
      deleteManhour(key: $key, mode: $mode) { key }
    }
"#;

const PERMISSION_QUERY: &str = r#"
    query Task($key: Key) {
      task(key: $key) {
        project { uuid }
        hasUpdateOwnAssessManHourPermission(attachPermission: {permissions: ["manage_task_own_assess_manhour"]})
        hasUpdateAllAssessManHourPermission(attachPermission: {permissions: ["manage_task_assess_manhour"]})
        hasUpdateOwnManHourPermission(attachPermission: {permissions: ["manage_task_own_record_manhours"]})
        hasUpdateAllManHourPermission(attachPermission: {permissions: ["manage_task_record_manhours"]})
      }
    }
"#;

impl OnesClient {
    pub(crate) async fn issue_manhours(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        owner_uuid: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> AppResult<ManhoursData> {
        let mut filter = serde_json::Map::new();
        filter.insert("task".to_string(), json!({"uuid_in": [issue_uuid]}));
        if let Some(owner_uuid) = owner_uuid {
            filter.insert("owner_in".to_string(), json!([owner_uuid]));
        }
        if let (Some(from), Some(to)) = (from, to) {
            filter.insert(
                "startTime_range".to_string(),
                json!({"unit": "day", "from": from, "to": to}),
            );
        }
        self.graphql(
            auth,
            "items/graphql?t=userManhours",
            ISSUE_MANHOURS_QUERY,
            json!({
                "manhourFilter": filter,
                "manhoursOrderBy": {"startTime": "DESC", "createTime": "DESC"},
            }),
        )
        .await
    }

    pub(crate) async fn issue_manhour_summary(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        owner_uuid: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> AppResult<ManhourSummaryData> {
        let mut filter = serde_json::Map::new();
        filter.insert("task".to_string(), json!({"uuid_equal": issue_uuid}));
        if let Some(owner_uuid) = owner_uuid {
            filter.insert("owner_equal".to_string(), json!(owner_uuid));
        }
        if let (Some(from), Some(to)) = (from, to) {
            filter.insert(
                "startTime_range".to_string(),
                json!({"unit": "day", "from": from, "to": to}),
            );
        }
        self.graphql(
            auth,
            "items/graphql?t=manhour-summary",
            ISSUE_SUMMARY_QUERY,
            json!({
                "groupBy": {"manhours": {"owner": {}, "task": {}}},
                "manhourFilter": filter,
                "orderBy": {"owner": {"namePinyin": "ASC"}},
            }),
        )
        .await
    }

    pub(crate) async fn work_hour_report(
        &self,
        auth: &AuthContext,
        from: &str,
        to: &str,
        owner_uuid: Option<&str>,
        issue_uuid: Option<&str>,
    ) -> AppResult<ManhourReportData> {
        let mut manhours_filter = serde_json::Map::new();
        manhours_filter.insert(
            "startTime_range".to_string(),
            json!({"unit": "day", "from": from, "to": to}),
        );
        if let Some(owner_uuid) = owner_uuid {
            manhours_filter.insert("owner_in".to_string(), json!([owner_uuid]));
        }
        if let Some(issue_uuid) = issue_uuid {
            manhours_filter.insert("task".to_string(), json!({"uuid_in": [issue_uuid]}));
        }
        self.graphql(
            auth,
            "items/graphql?t=report-data-manhour",
            REPORT_QUERY,
            json!({
                "groupBy": {"manhours": {"task": {}}},
                "orderBy": {"aggregateTask": {"createTime": "DESC"}},
                "filter": {
                    "manhours": manhours_filter,
                    "actualHours_notIn": [0],
                },
                "actualHoursSum": "manhours.recordedHour",
                "timeSeries": {
                    "timeField": "manhours.startTime",
                    "valueField": "manhours.recordedHour",
                    "unit": "day",
                    "from": from,
                    "to": to,
                },
                "columnSource": "task",
            }),
        )
        .await
    }

    pub(crate) async fn add_manhour(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        owner_uuid: &str,
        start_time: i64,
        hours: i64,
        description: &str,
    ) -> AppResult<String> {
        let response: ManhourMutationData = self
            .graphql(
                auth,
                "items/graphql?t=AddManhour",
                ADD_MANHOUR_MUTATION,
                json!({
                    "mode": "detailed",
                    "owner": owner_uuid,
                    "task": issue_uuid,
                    "type": "recorded",
                    "start_time": start_time,
                    "hours": hours,
                    "description": description,
                    "customData": {},
                }),
            )
            .await?;
        response
            .add
            .map(|result| result.key)
            .ok_or_else(|| AppError::invalid_upstream("addManhour returned no key"))
    }

    pub(crate) async fn update_remaining_hours(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
        owner_uuid: &str,
        hours: i64,
    ) -> AppResult<String> {
        let response: ManhourMutationData = self
            .graphql(
                auth,
                "items/graphql?t=UpdateRemainingHour",
                UPDATE_REMAINING_MUTATION,
                json!({
                    "mode": "detailed",
                    "key": format!("remaining_hour-{issue_uuid}-{owner_uuid}"),
                    "hours": hours,
                }),
            )
            .await?;
        response
            .update_remaining
            .map(|result| result.key)
            .ok_or_else(|| AppError::invalid_upstream("updateRemainingHour returned no key"))
    }

    pub(crate) async fn delete_manhour(&self, auth: &AuthContext, key: &str) -> AppResult<String> {
        let response: ManhourMutationData = self
            .graphql(
                auth,
                "items/graphql?t=DeleteManhour",
                DELETE_MANHOUR_MUTATION,
                json!({"key": key, "mode": "detailed"}),
            )
            .await?;
        response
            .delete
            .map(|result| result.key)
            .ok_or_else(|| AppError::invalid_upstream("deleteManhour returned no key"))
    }

    pub(crate) async fn task_manhour_permissions(
        &self,
        auth: &AuthContext,
        issue_uuid: &str,
    ) -> AppResult<TaskManhourPermissionData> {
        self.graphql(
            auth,
            "items/graphql?t=Task",
            PERMISSION_QUERY,
            json!({"key": format!("task-{issue_uuid}")}),
        )
        .await
    }

    async fn graphql<T>(
        &self,
        auth: &AuthContext,
        suffix: &str,
        query: &str,
        variables: Value,
    ) -> AppResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response: GraphqlResponse<T> = self
            .request_json(
                Method::POST,
                auth,
                self.project_team_url(suffix),
                Some(&json!({"query": query, "variables": variables})),
            )
            .await?;
        if !response.errors.is_empty() {
            return Err(AppError::Upstream(
                response
                    .errors
                    .into_iter()
                    .map(|error| error.message)
                    .collect::<Vec<_>>()
                    .join("; "),
            ));
        }
        response
            .data
            .ok_or_else(|| AppError::invalid_upstream("GraphQL response has no data"))
    }
}
