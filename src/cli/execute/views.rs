use super::super::{args::ViewCommand, args::ViewIssuesArgs, output};
use ones::{
    AppResult, OnesService,
    api::{GroupsResponse, ViewResponse, ViewsResponse},
    models::AuthContext,
};
use std::sync::Arc;

pub async fn run(
    service: Arc<OnesService>,
    auth: &AuthContext,
    command: ViewCommand,
    json: bool,
) -> AppResult<()> {
    match command {
        ViewCommand::List => {
            let value = ViewsResponse {
                views: service.list_workbench_views(auth).await?,
            };
            output::print_views(&value, json)
        }
        ViewCommand::Get { view_uuid } => {
            let value = ViewResponse {
                view: service.get_workbench_view(auth, &view_uuid).await?,
            };
            output::print_view(&value, json)
        }
        ViewCommand::Groups { view_uuid } => {
            let value = GroupsResponse {
                groups: service
                    .list_workbench_issue_groups(auth, &view_uuid)
                    .await?,
            };
            output::print_groups(&value, json)
        }
        ViewCommand::Issues(args) => list_issues(service, auth, args, json).await,
    }
}

pub async fn list_issues(
    service: Arc<OnesService>,
    auth: &AuthContext,
    args: ViewIssuesArgs,
    json: bool,
) -> AppResult<()> {
    let value = service
        .list_workbench_issues(
            auth,
            &args.view_uuid,
            args.group_value.as_deref(),
            args.cursor.as_deref(),
            args.limit,
            !args.no_redact,
        )
        .await?;
    output::print_issues(&value, json)
}
