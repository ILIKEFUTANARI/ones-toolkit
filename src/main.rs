mod cli;

use clap::Parser;
use ones::AppError;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_tracing();
    cli::run(cli::Cli::parse()).await
}

fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,rmcp=warn".into()))
        .with(fmt::layer())
        .try_init();
}
