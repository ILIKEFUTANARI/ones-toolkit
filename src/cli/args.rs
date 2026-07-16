use clap::{Args, Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(
    name = "ones",
    version,
    about = "ONES issue wrapper with HTTP API, MCP, and CLI"
)]
pub struct Cli {
    #[arg(long, global = true, help = "Print machine-readable JSON")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Serve {
        #[arg(long, env = "LISTEN_ADDR", default_value = "127.0.0.1:3000")]
        addr: SocketAddr,
        #[arg(long, env = "MCP_PATH", default_value = "/mcp")]
        mcp_path: String,
    },
    Stdio,
    Openapi,
    View {
        #[command(subcommand)]
        command: ViewCommand,
    },
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
    Get(DisplayIdArgs),
    Issues(ViewIssuesArgs),
    Attachments(IssueCursorArgs),
    Activity(IssueActivityArgs),
}

#[derive(Debug, Subcommand)]
pub enum ViewCommand {
    List,
    Get { view_uuid: String },
    Groups { view_uuid: String },
    Issues(ViewIssuesArgs),
}

#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    Get(IssueRedactionArgs),
    GetByDisplayId(DisplayIdArgs),
    Attachments(IssueCursorArgs),
    CodeLinks(IssueRedactionArgs),
    Activity(IssueActivityArgs),
    Transitions { issue_uuid: String },
    Branch(CreateBranchArgs),
    TransitionPreview(TransitionInputArgs),
    Transition(TransitionInputArgs),
}

#[derive(Debug, Args)]
pub struct DisplayIdArgs {
    pub display_id: String,
    #[arg(long)]
    pub no_redact: bool,
}

#[derive(Debug, Args)]
pub struct IssueRedactionArgs {
    pub issue_uuid: String,
    #[arg(long)]
    pub no_redact: bool,
}

#[derive(Debug, Args)]
pub struct IssueCursorArgs {
    pub issue_uuid: String,
    #[arg(long)]
    pub cursor: Option<String>,
}

#[derive(Debug, Args)]
pub struct IssueActivityArgs {
    pub issue_uuid: String,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub no_redact: bool,
}

#[derive(Debug, Args)]
pub struct ViewIssuesArgs {
    pub view_uuid: String,
    #[arg(long)]
    pub group_value: Option<String>,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub no_redact: bool,
}

#[derive(Debug, Args)]
pub struct CreateBranchArgs {
    pub issue_uuid: String,
    #[arg(long)]
    pub repo_uuid: String,
    #[arg(long)]
    pub source_branch: String,
    #[arg(long)]
    pub target_branch: String,
}

#[derive(Debug, Args)]
pub struct TransitionInputArgs {
    pub issue_uuid: String,
    #[arg(
        long,
        value_name = "PATH|-",
        help = "Read transition JSON from file or stdin"
    )]
    pub input: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_server_defaults() {
        let cli = Cli::try_parse_from(["ones", "serve"]).unwrap();
        let Command::Serve { addr, mcp_path } = cli.command else {
            panic!("expected serve command");
        };
        assert_eq!(addr, "127.0.0.1:3000".parse().unwrap());
        assert_eq!(mcp_path, "/mcp");
    }

    #[test]
    fn parses_shortcuts_and_json() {
        let cli = Cli::try_parse_from([
            "ones",
            "--json",
            "attachments",
            "issue-demo-001",
            "--cursor",
            "100",
        ])
        .unwrap();
        assert!(cli.json);
        let Command::Attachments(args) = cli.command else {
            panic!("expected attachment shortcut");
        };
        assert_eq!(args.issue_uuid, "issue-demo-001");
        assert_eq!(args.cursor.as_deref(), Some("100"));
    }

    #[test]
    fn parses_transition_stdin_input() {
        let cli = Cli::try_parse_from([
            "ones",
            "issue",
            "transition-preview",
            "issue-demo-001",
            "--input",
            "-",
        ])
        .unwrap();
        let Command::Issue {
            command: IssueCommand::TransitionPreview(args),
        } = cli.command
        else {
            panic!("expected transition preview");
        };
        assert_eq!(args.input, "-");
    }
}
