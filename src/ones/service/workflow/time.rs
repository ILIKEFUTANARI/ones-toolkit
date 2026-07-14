use crate::error::{AppError, AppResult};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn parse_start_time(value: Option<&str>) -> AppResult<i64> {
    match value {
        None => Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
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
    if date.len() != 3
        || !(1..=12).contains(&date[1])
        || !(1..=31).contains(&date[2])
        || time[0] > 23
        || time[1] > 59
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
