use super::super::{
    args::{
        CreateBranchArgs, DisplayIdArgs, IssueActivityArgs, IssueCommand, IssueCursorArgs,
        IssueRedactionArgs, TransitionInputArgs,
    },
    output,
};
use ones::{
    AppError, AppResult, OnesService,
    api::{AttachmentsResponse, IssueResponse, LegacyTaskInfoResponse},
    code::CreateBranchInput,
    models::AuthContext,
    workflow::TransitionValues,
};
use serde_json::from_str;
use std::{
    fs,
    io::{self, Read},
    sync::Arc,
};

pub async fn run(
    service: Arc<OnesService>,
    auth: &AuthContext,
    command: IssueCommand,
    json: bool,
) -> AppResult<()> {
    match command {
        IssueCommand::Get(args) => get(service, auth, args, json).await,
        IssueCommand::GetByDisplayId(args) => get_by_display_id(service, auth, &args, json).await,
        IssueCommand::TaskInfo { issue_uuid } => task_info(service, auth, &issue_uuid, json).await,
        IssueCommand::Attachments(args) => attachments(service, auth, args, json).await,
        IssueCommand::CodeLinks(args) => code_links(service, auth, args, json).await,
        IssueCommand::Activity(args) => activity(service, auth, args, json).await,
        IssueCommand::Transitions { issue_uuid } => {
            transitions(service, auth, &issue_uuid, json).await
        }
        IssueCommand::Branch(args) => branch(service, auth, args, json).await,
        IssueCommand::TransitionPreview(args) => {
            transition_preview(service, auth, args, json).await
        }
        IssueCommand::Transition(args) => transition(service, auth, args, json).await,
    }
}

async fn get(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: IssueRedactionArgs,
    json: bool,
) -> AppResult<()> {
    let value = IssueResponse {
        issue: service
            .get_issue_detail(auth, &args.issue_uuid, !args.no_redact)
            .await?,
    };
    output::print_issue(&value, json)
}

pub async fn get_by_display_id(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: &DisplayIdArgs,
    json: bool,
) -> AppResult<()> {
    let value = IssueResponse {
        issue: service
            .get_issue_by_display_id(auth, &args.display_id, !args.no_redact)
            .await?,
    };
    output::print_issue(&value, json)
}

pub async fn task_info(
    service: Arc<OnesService>,
    auth: &AuthContext,
    issue_uuid: &str,
    json: bool,
) -> AppResult<()> {
    let value = LegacyTaskInfoResponse {
        task_info: service.legacy_task_info(auth, issue_uuid).await?,
    };
    output::print_legacy_task_info(&value, json)
}

pub async fn attachments(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: IssueCursorArgs,
    json: bool,
) -> AppResult<()> {
    let value: AttachmentsResponse = service
        .list_issue_attachments(auth, &args.issue_uuid, args.cursor.as_deref())
        .await?;
    output::print_attachments(&value, json)
}

async fn code_links(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: IssueRedactionArgs,
    json: bool,
) -> AppResult<()> {
    let value = service
        .list_issue_code_links(auth, &args.issue_uuid, !args.no_redact)
        .await?;
    output::print_code_links(&value, json)
}

pub async fn activity(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: IssueActivityArgs,
    json: bool,
) -> AppResult<()> {
    let value = service
        .list_issue_activity(
            auth,
            &args.issue_uuid,
            args.cursor.as_deref(),
            args.limit,
            !args.no_redact,
        )
        .await?;
    output::print_activity(&value, json)
}

async fn transitions(
    service: Arc<OnesService>,
    auth: &AuthContext,
    issue_uuid: &str,
    json: bool,
) -> AppResult<()> {
    let value = service.list_issue_transitions(auth, issue_uuid).await?;
    output::print_transitions(&value, json)
}

async fn branch(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: CreateBranchArgs,
    json: bool,
) -> AppResult<()> {
    let input = CreateBranchInput {
        repo_uuid: args.repo_uuid,
        source_branch: args.source_branch,
        target_branch: args.target_branch,
    };
    let value = service
        .create_issue_branch(auth, &args.issue_uuid, &input)
        .await?;
    output::print_branch(&value, json)
}

async fn transition_preview(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: TransitionInputArgs,
    json: bool,
) -> AppResult<()> {
    let values = read_transition_values(&args.input)?;
    let value = service
        .preview_issue_transition(auth, &args.issue_uuid, &values)
        .await?;
    output::print_details(&value, json)
}

async fn transition(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: TransitionInputArgs,
    json: bool,
) -> AppResult<()> {
    let values = read_transition_values(&args.input)?;
    let value = service
        .execute_issue_transition(auth, &args.issue_uuid, &values)
        .await?;
    output::print_details(&value, json)
}

fn read_transition_values(input: &str) -> AppResult<TransitionValues> {
    let mut body = String::new();
    if input == "-" {
        io::stdin().read_to_string(&mut body)?;
    } else {
        body = fs::read_to_string(input)?;
    }
    from_str(&body).map_err(AppError::from)
}
