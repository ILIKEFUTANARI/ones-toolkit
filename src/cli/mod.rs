mod args;
mod execute;
mod output;

pub use args::Cli;

pub async fn run(cli: Cli) -> ones::AppResult<()> {
    execute::run(cli).await
}
