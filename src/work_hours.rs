use crate::error::{AppError, AppResult};
use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueSelector {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueReference {
    pub issue_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WorkHourType {
    Recorded,
    Estimated,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourOwner {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourEntry {
    pub key: String,
    pub hours: f64,
    pub start_at: String,
    pub description: String,
    #[serde(rename = "type")]
    pub hour_type: WorkHourType,
    pub owner: WorkHourOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourSummary {
    pub owner: WorkHourOwner,
    pub actual_hours: f64,
    pub estimated_hours: f64,
    pub remaining_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct IssueWorkHoursResponse {
    pub issue: IssueReference,
    pub entries: Vec<WorkHourEntry>,
    pub summaries: Vec<WorkHourSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourSeriesPoint {
    pub date: String,
    pub hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourReportTask {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourReportBucket {
    pub task: WorkHourReportTask,
    pub actual_hours: f64,
    pub series: Vec<WorkHourSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourReportResponse {
    pub from: String,
    pub to: String,
    pub buckets: Vec<WorkHourReportBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ListIssueWorkHoursQuery {
    pub issue: IssueSelector,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redact: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourReportQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<IssueSelector>,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redact: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct AddWorkHourInput {
    pub issue: IssueSelector,
    pub hours: f64,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct UpdateRemainingHoursInput {
    pub issue: IssueSelector,
    pub remaining_hours: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct DeleteWorkHourInput {
    pub issue: IssueSelector,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct WorkHourMutationResponse {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_type: Option<WorkHourType>,
}

pub(crate) const ONES_HOURS_UNIT: f64 = 100_000.0;

pub(crate) fn hours_from_units(units: i64) -> f64 {
    units as f64 / ONES_HOURS_UNIT
}

pub(crate) fn hours_to_units(hours: f64, field: &str, allow_zero: bool) -> AppResult<i64> {
    let valid = hours.is_finite() && (allow_zero && hours >= 0.0 || !allow_zero && hours > 0.0);
    if !valid {
        let condition = if allow_zero {
            "non-negative"
        } else {
            "positive"
        };
        return Err(AppError::invalid_input(format!(
            "{field} must be a finite {condition} number"
        )));
    }
    let units = (hours * ONES_HOURS_UNIT).round();
    if !units.is_finite() || units > i64::MAX as f64 || units < 0.0 {
        return Err(AppError::invalid_input(format!("{field} is out of range")));
    }
    let units = units as i64;
    if !allow_zero && units <= 0 {
        return Err(AppError::invalid_input(format!(
            "{field} is below the ONES precision"
        )));
    }
    Ok(units)
}

pub(crate) fn parse_start_time(value: Option<&str>) -> AppResult<i64> {
    match value {
        None => Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| AppError::invalid_input("system clock is before unix epoch"))?
            .as_secs() as i64),
        Some(value) => parse_rfc3339_utc(value),
    }
}

fn parse_rfc3339_utc(value: &str) -> AppResult<i64> {
    let (date, time) = value
        .strip_suffix('Z')
        .and_then(|value| value.split_once('T'))
        .ok_or_else(|| AppError::invalid_input("start_at must be RFC3339 UTC ending in Z"))?;
    let date_parts = date.split('-').collect::<Vec<_>>();
    let time_parts = time.split(':').collect::<Vec<_>>();
    if date_parts.len() != 3
        || date_parts[0].len() != 4
        || date_parts[1].len() != 2
        || date_parts[2].len() != 2
        || time_parts.len() != 3
        || time_parts[0].len() != 2
        || time_parts[1].len() != 2
    {
        return Err(AppError::invalid_input("start_at has invalid date or time"));
    }
    let date = date_parts
        .iter()
        .map(|part| part.parse::<i64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AppError::invalid_input("start_at has invalid date"))?;
    let seconds = time_parts[2]
        .split_once('.')
        .map(|(seconds, fraction)| {
            if fraction.is_empty() || !fraction.chars().all(|character| character.is_ascii_digit())
            {
                Err(())
            } else {
                seconds.parse::<i64>().map_err(|_| ())
            }
        })
        .unwrap_or_else(|| time_parts[2].parse::<i64>().map_err(|_| ()))
        .map_err(|_| AppError::invalid_input("start_at has invalid time"))?;
    let time = [
        time_parts[0]
            .parse::<i64>()
            .map_err(|_| AppError::invalid_input("start_at has invalid time"))?,
        time_parts[1]
            .parse::<i64>()
            .map_err(|_| AppError::invalid_input("start_at has invalid time"))?,
        seconds,
    ];
    if !(1..=12).contains(&date[1])
        || !(1..=31).contains(&date[2])
        || time[0] < 0
        || time[0] > 23
        || time[1] < 0
        || time[1] > 59
        || time[2] < 0
        || time[2] > 59
        || days_in_month(date[0], date[1]) < date[2]
    {
        return Err(AppError::invalid_input("start_at has invalid date or time"));
    }
    Ok(days_from_civil(date[0], date[1], date[2]) * 86_400
        + time[0] * 3_600
        + time[1] * 60
        + time[2])
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - (month <= 2) as i64;
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::{hours_to_units, parse_start_time};

    #[test]
    fn converts_hours_at_ones_precision() {
        assert_eq!(hours_to_units(1.25, "hours", false).unwrap(), 125_000);
        assert_eq!(hours_to_units(0.0, "remaining_hours", true).unwrap(), 0);
        assert!(hours_to_units(0.0, "hours", false).is_err());
    }

    #[test]
    fn accepts_utc_start_time_and_rejects_invalid_clock_values() {
        assert_eq!(
            parse_start_time(Some("2026-07-20T09:00:00Z")).unwrap(),
            1_784_538_000
        );
        assert!(parse_start_time(Some("2026-07-20T-1:00:00Z")).is_err());
        assert!(parse_start_time(Some("2026-07-20T09:-1:00Z")).is_err());
        assert!(parse_start_time(Some("2026-07-20T09:00:-1Z")).is_err());
    }
}
