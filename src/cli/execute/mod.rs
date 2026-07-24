mod issues;
mod views;
mod work_hours;

use super::{
    args::{Cli, Command},
    output,
};
use ones::{
    AppConfig, AppResult, McpAuthMode, OnesMcpServer, OnesService, StdioAuthConfig, build_http_app,
};
use rmcp::ServiceExt;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

pub async fn run(cli: Cli) -> AppResult<()> {
    let Cli { json, command } = cli;
    match command {
        Command::Openapi => output::print_json(&ones::openapi_document()),
        Command::Serve { addr, mcp_path } => serve(addr, mcp_path).await,
        Command::Stdio => stdio().await,
        command => {
            let config = AppConfig::from_env()?;
            let auth = StdioAuthConfig::from_env()?.into_auth_context();
            let service = Arc::new(OnesService::new(config));
            run_business_command(service, auth, command, json).await
        }
    }
}

async fn serve(addr: SocketAddr, mcp_path: String) -> AppResult<()> {
    let config = AppConfig::from_env()?;
    let service = Arc::new(OnesService::new(config));
    let app = build_http_app(service, McpAuthMode::HttpHeaders, mcp_path);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn stdio() -> AppResult<()> {
    let config = AppConfig::from_env()?;
    let service = Arc::new(OnesService::new(config));
    let auth = StdioAuthConfig::from_env()?;
    let server = OnesMcpServer::new(service, McpAuthMode::Static(auth.into_auth_context()));
    server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|error| ones::AppError::Upstream(error.to_string()))?;
    Ok(())
}

async fn run_business_command(
    service: Arc<OnesService>,
    auth: ones::models::AuthContext,
    command: Command,
    json: bool,
) -> AppResult<()> {
    match command {
        Command::View { command } => views::run(service, &auth, command, json).await,
        Command::Issue { command } => issues::run(service, &auth, command, json).await,
        Command::Get(args) => issues::get_by_display_id(service, &auth, &args, json).await,
        Command::Issues(args) => views::list_issues(service, &auth, args, json).await,
        Command::Attachments(args) => issues::attachments(service, &auth, args, json).await,
        Command::Activity(args) => issues::activity(service, &auth, args, json).await,
        Command::WorkHour { command } => work_hours::run(service, &auth, command, json).await,
        Command::Serve { .. } | Command::Stdio | Command::Openapi => {
            unreachable!("non-business command handled before service creation")
        }
    }
}
