#![cfg(feature = "cli")]

use std::process::Command;

#[test]
fn openapi_command_prints_generated_document() {
    let output = Command::new(env!("CARGO_BIN_EXE_ones"))
        .args(["openapi"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["info"]["title"], "ONES Toolkit (Unofficial)");
    assert!(document["paths"]["/api/issues/{issue_uuid}/transition"].is_object());
    assert!(document["paths"]["/api/issues/{issue_uuid}/task-info"].is_object());
}

#[test]
fn help_lists_shortcut_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_ones"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    for command in [
        "get",
        "issues",
        "attachments",
        "activity",
        "work-hour",
        "openapi",
    ] {
        assert!(help.contains(command), "missing shortcut {command}: {help}");
    }
}

#[test]
fn issue_help_lists_task_info_subcommand() {
    let output = Command::new(env!("CARGO_BIN_EXE_ones"))
        .args(["issue", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success());
    assert!(
        help.contains("task-info"),
        "missing issue subcommand task-info: {help}"
    );
}

#[test]
fn work_hour_help_lists_unified_operations() {
    let output = Command::new(env!("CARGO_BIN_EXE_ones"))
        .args(["work-hour", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    for command in ["list", "report", "add", "update-remaining", "delete"] {
        assert!(
            help.contains(command),
            "missing work-hour command {command}: {help}"
        );
    }
}
