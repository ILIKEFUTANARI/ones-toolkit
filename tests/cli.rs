use std::process::Command;

#[test]
fn openapi_command_prints_generated_document() {
    let output = Command::new(env!("CARGO_BIN_EXE_ones"))
        .args(["openapi"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["info"]["title"], "ONES Issue Wrapper");
    assert!(document["paths"]["/api/issues/{issue_uuid}/transition"].is_object());
}

#[test]
fn help_lists_shortcut_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_ones"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    for command in ["get", "issues", "attachments", "activity", "openapi"] {
        assert!(help.contains(command), "missing shortcut {command}: {help}");
    }
}
