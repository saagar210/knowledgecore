use std::process::Command;

fn kc_cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_kc_cli")
}

#[cfg(not(feature = "phase_l_preview"))]
#[test]
fn preview_command_is_not_available_by_default() {
    let out = Command::new(kc_cli_bin())
        .args(["preview", "status"])
        .output()
        .expect("run kc_cli");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("preview"));
}

#[cfg(feature = "phase_l_preview")]
#[test]
fn preview_status_returns_draft_capability_matrix() {
    let out = Command::new(kc_cli_bin())
        .args(["preview", "status"])
        .output()
        .expect("run kc_cli");

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"status\": \"draft\""));
    assert!(stdout.contains("\"capability\": \"encryption\""));
    assert!(stdout.contains("\"capability\": \"zip_packaging\""));
}

#[cfg(feature = "phase_l_preview")]
#[test]
fn preview_capability_returns_draft_error_code() {
    let out = Command::new(kc_cli_bin())
        .args(["preview", "capability", "--name", "sync"])
        .output()
        .expect("run kc_cli");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("KC_DRAFT_SYNC_NOT_IMPLEMENTED"));
}
