use clap::{Parser, Subcommand};
use ones::{
    AppConfig, AppError, McpAuthMode, OnesMcpServer, OnesService, StdioAuthConfig, build_http_app,
};
use rmcp::ServiceExt;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Debug, Parser)]
#[command(
    name = "ones",
    version,
    about = "ONES issue wrapper with HTTP API and MCP"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Serve {
        #[arg(long, env = "LISTEN_ADDR", default_value = "127.0.0.1:3000")]
        addr: SocketAddr,
        #[arg(long, env = "MCP_PATH", default_value = "/mcp")]
        mcp_path: String,
    },
    Stdio,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_tracing();

    let cli = Cli::parse();
    let config = AppConfig::from_env()?;
    let service = Arc::new(OnesService::new(config.clone()));

    match cli.command {
        Command::Serve { addr, mcp_path } => {
            let app = build_http_app(service, McpAuthMode::HttpHeaders, mcp_path);
            let listener = TcpListener::bind(addr).await?;
            tracing::info!(%addr, "listening");
            axum::serve(listener, app).await?;
        }
        Command::Stdio => {
            let auth = StdioAuthConfig::from_env()?;
            let server = OnesMcpServer::new(service, McpAuthMode::Static(auth.into_auth_context()));
            server
                .serve(rmcp::transport::stdio())
                .await
                .map_err(|error| AppError::Upstream(error.to_string()))?;
        }
    }

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,rmcp=warn".into()))
        .with(fmt::layer())
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serve_defaults_to_local_port_3000() {
        let cli = Cli::try_parse_from(["ones", "serve"]).unwrap();
        let Command::Serve { addr, mcp_path } = cli.command else {
            panic!("expected serve command");
        };
        assert_eq!(addr, "127.0.0.1:3000".parse().unwrap());
        assert_eq!(mcp_path, "/mcp");
    }
}
