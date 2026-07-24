use super::super::{
    args::{
        WorkHourCommand, WorkHourDeleteArgs, WorkHourListArgs, WorkHourReportArgs,
        WorkHourUpdateArgs,
    },
    output,
};
use ones::{
    AppError, AppResult, OnesService,
    models::AuthContext,
    work_hours::{
        AddWorkHourInput, DeleteWorkHourInput, ListIssueWorkHoursQuery, UpdateRemainingHoursInput,
        WorkHourReportQuery,
    },
};
use serde::de::DeserializeOwned;
use std::{
    fs,
    io::{self, Read},
    sync::Arc,
};

pub async fn run(
    service: Arc<OnesService>,
    auth: &AuthContext,
    command: WorkHourCommand,
    json: bool,
) -> AppResult<()> {
    match command {
        WorkHourCommand::List(args) => list(service, auth, args, json).await,
        WorkHourCommand::Report(args) => report(service, auth, args, json).await,
        WorkHourCommand::Add { input } => {
            let input: AddWorkHourInput = read_json(&input)?;
            output::print_details(&service.add_work_hour(auth, &input).await?, json)
        }
        WorkHourCommand::UpdateRemaining(args) => update_remaining(service, auth, args, json).await,
        WorkHourCommand::Delete(args) => delete(service, auth, args, json).await,
    }
}

async fn list(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: WorkHourListArgs,
    json: bool,
) -> AppResult<()> {
    let response = service
        .list_issue_work_hours(
            auth,
            &ListIssueWorkHoursQuery {
                issue: args.selector.selector(),
                from: args.from,
                to: args.to,
                owner_uuid: args.owner_uuid,
                redact: Some(!args.no_redact),
            },
        )
        .await?;
    output::print_details(&response, json)
}

async fn report(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: WorkHourReportArgs,
    json: bool,
) -> AppResult<()> {
    let selector = args.selector.selector();
    let issue =
        (selector.issue_uuid.is_some() || selector.display_id.is_some()).then_some(selector);
    let response = service
        .query_work_hour_report(
            auth,
            &WorkHourReportQuery {
                issue,
                from: args.from,
                to: args.to,
                owner_uuid: args.owner_uuid,
                redact: Some(!args.no_redact),
            },
        )
        .await?;
    output::print_details(&response, json)
}

async fn update_remaining(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: WorkHourUpdateArgs,
    json: bool,
) -> AppResult<()> {
    let response = service
        .update_remaining_hours(
            auth,
            &UpdateRemainingHoursInput {
                issue: args.selector.selector(),
                remaining_hours: args.remaining_hours,
                owner_uuid: args.owner_uuid,
            },
        )
        .await?;
    output::print_details(&response, json)
}

async fn delete(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: WorkHourDeleteArgs,
    json: bool,
) -> AppResult<()> {
    let response = service
        .delete_work_hour(
            auth,
            &DeleteWorkHourInput {
                issue: args.selector.selector(),
                key: args.key,
            },
        )
        .await?;
    output::print_details(&response, json)
}

fn read_json<T: DeserializeOwned>(input: &str) -> AppResult<T> {
    let mut body = String::new();
    if input == "-" {
        io::stdin().read_to_string(&mut body)?;
    } else {
        body = fs::read_to_string(input)?;
    }
    serde_json::from_str(&body).map_err(AppError::from)
}
