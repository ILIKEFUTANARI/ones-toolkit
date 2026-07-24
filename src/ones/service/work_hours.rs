use super::OnesService;
use crate::{
    error::{AppError, AppResult},
    models::AuthContext,
    ones::{
        transform::format_timestamp,
        work_hour_raw::{ManhourOwner, ManhourRecord, TaskManhourPermission},
    },
    work_hours::{
        AddWorkHourInput, DeleteWorkHourInput, IssueReference, IssueSelector,
        IssueWorkHoursResponse, ListIssueWorkHoursQuery, UpdateRemainingHoursInput, WorkHourEntry,
        WorkHourMutationResponse, WorkHourOwner, WorkHourReportBucket, WorkHourReportQuery,
        WorkHourReportResponse, WorkHourReportTask, WorkHourSeriesPoint, WorkHourSummary,
        WorkHourType, hours_from_units, hours_to_units, parse_start_time,
    },
};
use serde_json::json;

struct ResolvedIssue {
    uuid: String,
    display_id: Option<String>,
}

impl OnesService {
    pub async fn list_issue_work_hours(
        &self,
        auth: &AuthContext,
        query: &ListIssueWorkHoursQuery,
    ) -> AppResult<IssueWorkHoursResponse> {
        let issue = self.resolve_issue(auth, &query.issue).await?;
        validate_date_range(query.from.as_deref(), query.to.as_deref())?;
        let redact = query.redact.unwrap_or(true);
        let (entries, summary) = tokio::try_join!(
            self.client.issue_manhours(
                auth,
                &issue.uuid,
                query.owner_uuid.as_deref(),
                query.from.as_deref(),
                query.to.as_deref(),
            ),
            self.client.issue_manhour_summary(
                auth,
                &issue.uuid,
                query.owner_uuid.as_deref(),
                query.from.as_deref(),
                query.to.as_deref(),
            ),
        )?;
        Ok(IssueWorkHoursResponse {
            issue: IssueReference {
                issue_uuid: issue.uuid,
                display_id: issue.display_id,
            },
            entries: self.map_entries(entries.manhours, redact)?,
            summaries: self.map_summaries(summary.buckets, redact),
        })
    }

    pub async fn query_work_hour_report(
        &self,
        auth: &AuthContext,
        query: &WorkHourReportQuery,
    ) -> AppResult<WorkHourReportResponse> {
        validate_date_range(Some(&query.from), Some(&query.to))?;
        let issue_uuid = match query.issue.as_ref() {
            Some(selector) => Some(self.resolve_issue(auth, selector).await?.uuid),
            None => None,
        };
        let report = self
            .client
            .work_hour_report(
                auth,
                &query.from,
                &query.to,
                query.owner_uuid.as_deref(),
                issue_uuid.as_deref(),
            )
            .await?;
        let buckets = report
            .buckets
            .into_iter()
            .filter(|bucket| {
                issue_uuid
                    .as_deref()
                    .is_none_or(|issue_uuid| issue_uuid == bucket.task.uuid)
            })
            .map(|bucket| {
                if bucket.series.times.len() != bucket.series.values.len() {
                    return Err(AppError::invalid_upstream(
                        "actualHoursSeries times and values have different lengths",
                    ));
                }
                Ok(WorkHourReportBucket {
                    task: WorkHourReportTask {
                        uuid: bucket.task.uuid,
                        display_id: bucket.task.display_id,
                        title: bucket.task.name,
                        project_uuid: bucket.task.project.map(|project| project.uuid),
                    },
                    actual_hours: hours_from_units(bucket.actual_hours),
                    series: bucket
                        .series
                        .times
                        .into_iter()
                        .zip(bucket.series.values)
                        .map(|(date, hours)| WorkHourSeriesPoint {
                            date,
                            hours: hours_from_units(hours),
                        })
                        .collect(),
                })
            })
            .collect::<AppResult<Vec<_>>>()?;
        Ok(WorkHourReportResponse {
            from: query.from.clone(),
            to: query.to.clone(),
            buckets,
        })
    }

    pub async fn add_work_hour(
        &self,
        auth: &AuthContext,
        input: &AddWorkHourInput,
    ) -> AppResult<WorkHourMutationResponse> {
        let issue = self.resolve_issue(auth, &input.issue).await?;
        if input.description.trim().is_empty() {
            return Err(AppError::invalid_input("description must be non-empty"));
        }
        let units = hours_to_units(input.hours, "hours", false)?;
        let start_time = parse_start_time(input.start_at.as_deref())?;
        let current_user = self.client.current_user_uuid(auth).await?;
        let owner_uuid = input.owner_uuid.as_deref().unwrap_or(&current_user);
        let permissions = self
            .client
            .task_manhour_permissions(auth, &issue.uuid)
            .await?;
        ensure_permission(
            &permissions.task,
            owner_uuid,
            &current_user,
            &WorkHourType::Recorded,
        )?;
        let limit = self
            .client
            .recordable_manhour(auth, &permissions.task.project.uuid, start_time, owner_uuid)
            .await?
            .limit;
        if units > limit {
            return Err(AppError::invalid_input(format!(
                "hours exceeds recordable limit {limit}"
            )));
        }
        let key = self
            .client
            .add_manhour(
                auth,
                &issue.uuid,
                owner_uuid,
                start_time,
                units,
                &input.description,
            )
            .await?;
        Ok(WorkHourMutationResponse {
            key,
            remaining_hours: None,
            deleted_type: None,
        })
    }

    pub async fn update_remaining_hours(
        &self,
        auth: &AuthContext,
        input: &UpdateRemainingHoursInput,
    ) -> AppResult<WorkHourMutationResponse> {
        let issue = self.resolve_issue(auth, &input.issue).await?;
        let units = hours_to_units(input.remaining_hours, "remaining_hours", true)?;
        let current_user = self.client.current_user_uuid(auth).await?;
        let owner_uuid = input.owner_uuid.as_deref().unwrap_or(&current_user);
        let permissions = self
            .client
            .task_manhour_permissions(auth, &issue.uuid)
            .await?;
        ensure_permission(
            &permissions.task,
            owner_uuid,
            &current_user,
            &WorkHourType::Estimated,
        )?;
        let key = self
            .client
            .update_remaining_hours(auth, &issue.uuid, owner_uuid, units)
            .await?;
        Ok(WorkHourMutationResponse {
            key,
            remaining_hours: Some(input.remaining_hours),
            deleted_type: None,
        })
    }

    pub async fn delete_work_hour(
        &self,
        auth: &AuthContext,
        input: &DeleteWorkHourInput,
    ) -> AppResult<WorkHourMutationResponse> {
        if input.key.trim().is_empty() {
            return Err(AppError::invalid_input("key must be non-empty"));
        }
        let issue = self.resolve_issue(auth, &input.issue).await?;
        let entries = self
            .client
            .issue_manhours(auth, &issue.uuid, None, None, None)
            .await?
            .manhours;
        let entry = entries
            .into_iter()
            .find(|entry| entry.key == input.key)
            .ok_or_else(|| AppError::NotFound(format!("work hour {}", input.key)))?;
        let hour_type = parse_hour_type(&entry.hour_type)?;
        let current_user = self.client.current_user_uuid(auth).await?;
        let permissions = self
            .client
            .task_manhour_permissions(auth, &issue.uuid)
            .await?;
        ensure_permission(
            &permissions.task,
            &entry.owner.uuid,
            &current_user,
            &hour_type,
        )?;
        let key = self.client.delete_manhour(auth, &input.key).await?;
        Ok(WorkHourMutationResponse {
            key,
            remaining_hours: None,
            deleted_type: Some(hour_type),
        })
    }

    async fn resolve_issue(
        &self,
        auth: &AuthContext,
        selector: &IssueSelector,
    ) -> AppResult<ResolvedIssue> {
        match (
            selector
                .issue_uuid
                .as_deref()
                .filter(|value| !value.trim().is_empty()),
            selector
                .display_id
                .as_deref()
                .filter(|value| !value.trim().is_empty()),
        ) {
            (Some(_), Some(_)) => Err(AppError::invalid_input(
                "provide exactly one of issue_uuid or display_id",
            )),
            (Some(issue_uuid), None) => Ok(ResolvedIssue {
                uuid: issue_uuid.to_string(),
                display_id: None,
            }),
            (None, Some(display_id)) => {
                let resolved = self.client.resolve_display_id(auth, display_id).await?;
                let issue_uuid = resolved
                    .task_uuid
                    .ok_or_else(|| AppError::NotFound(format!("display id {display_id}")))?;
                Ok(ResolvedIssue {
                    uuid: issue_uuid,
                    display_id: resolved.display_id.or(Some(display_id.to_string())),
                })
            }
            (None, None) => Err(AppError::invalid_input(
                "one of issue_uuid or display_id is required",
            )),
        }
    }

    fn map_entries(
        &self,
        entries: Vec<ManhourRecord>,
        redact: bool,
    ) -> AppResult<Vec<WorkHourEntry>> {
        let mut identities = self.identities.scope(redact);
        entries
            .into_iter()
            .map(|entry| {
                let hour_type = parse_hour_type(&entry.hour_type)?;
                let milliseconds = entry
                    .start_time
                    .checked_mul(1000)
                    .ok_or_else(|| AppError::invalid_upstream("work hour timestamp overflow"))?;
                let start_at = format_timestamp(&json!(milliseconds))
                    .ok_or_else(|| AppError::invalid_upstream("invalid work hour timestamp"))?;
                Ok(WorkHourEntry {
                    key: entry.key,
                    hours: hours_from_units(entry.hours),
                    start_at,
                    description: entry.description,
                    hour_type,
                    owner: map_owner(entry.owner, &mut identities),
                })
            })
            .collect()
    }

    fn map_summaries(
        &self,
        summaries: Vec<crate::ones::work_hour_raw::ManhourSummaryBucket>,
        redact: bool,
    ) -> Vec<WorkHourSummary> {
        let mut identities = self.identities.scope(redact);
        summaries
            .into_iter()
            .map(|summary| WorkHourSummary {
                owner: map_owner(summary.owner, &mut identities),
                actual_hours: hours_from_units(summary.actual_hours),
                estimated_hours: hours_from_units(summary.estimated_hours),
                remaining_hours: hours_from_units(summary.remaining_hours),
            })
            .collect()
    }
}

fn map_owner(
    owner: ManhourOwner,
    identities: &mut crate::ones::identity::IdentityScope<'_>,
) -> WorkHourOwner {
    WorkHourOwner {
        uuid: owner.uuid.clone(),
        name: identities.person_parts(Some(&owner.uuid), owner.name.as_deref()),
    }
}

fn parse_hour_type(value: &str) -> AppResult<WorkHourType> {
    match value {
        "recorded" => Ok(WorkHourType::Recorded),
        "estimated" => Ok(WorkHourType::Estimated),
        other => Err(AppError::invalid_upstream(format!(
            "unsupported ONES work hour type {other}"
        ))),
    }
}

fn ensure_permission(
    permission: &TaskManhourPermission,
    owner_uuid: &str,
    current_user_uuid: &str,
    hour_type: &WorkHourType,
) -> AppResult<()> {
    let is_current_user = owner_uuid == current_user_uuid;
    let allowed = match (hour_type, is_current_user) {
        (WorkHourType::Recorded, true) => {
            permission.update_own_recorded || permission.update_all_recorded
        }
        (WorkHourType::Recorded, false) => permission.update_all_recorded,
        (WorkHourType::Estimated, true) => {
            permission.update_own_assess || permission.update_all_assess
        }
        (WorkHourType::Estimated, false) => permission.update_all_assess,
    };
    if allowed {
        Ok(())
    } else {
        let category = match hour_type {
            WorkHourType::Recorded => "recorded",
            WorkHourType::Estimated => "estimated",
        };
        Err(AppError::invalid_input(format!(
            "no permission to modify {category} work hours for owner {owner_uuid}"
        )))
    }
}

fn validate_date_range(from: Option<&str>, to: Option<&str>) -> AppResult<()> {
    match (from, to) {
        (None, None) => Ok(()),
        (Some(_), None) | (None, Some(_)) => Err(AppError::invalid_input(
            "from and to must be provided together",
        )),
        (Some(from), Some(to)) => {
            parse_date(from)?;
            parse_date(to)?;
            if from > to {
                return Err(AppError::invalid_input("from must not be after to"));
            }
            Ok(())
        }
    }
}

fn parse_date(value: &str) -> AppResult<()> {
    parse_start_time(Some(&format!("{value}T00:00:00Z")))?;
    Ok(())
}
