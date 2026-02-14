use kc_core::vault::vault_init;
use std::process::Command;

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kc_cli"))
        .args(args)
        .output()
        .expect("run kc_cli")
}

#[test]
fn cli_trust_identity_and_device_workflow_round_trip() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let vault_path = root.to_string_lossy().to_string();

    let identity_start = run_cli(&[
        "trust",
        "identity",
        "start",
        &vault_path,
        "--provider",
        "default",
        "--now-ms",
        "10",
    ]);
    assert!(
        identity_start.status.success(),
        "identity start stderr: {}",
        String::from_utf8_lossy(&identity_start.stderr)
    );

    let identity_complete = run_cli(&[
        "trust",
        "identity",
        "complete",
        &vault_path,
        "--provider",
        "default",
        "--code",
        "sub:alice@example.com",
        "--now-ms",
        "11",
    ]);
    assert!(
        identity_complete.status.success(),
        "identity complete stderr: {}",
        String::from_utf8_lossy(&identity_complete.stderr)
    );

    let device_enroll = run_cli(&[
        "trust",
        "device",
        "enroll",
        &vault_path,
        "--device-label",
        "desktop",
        "--now-ms",
        "12",
    ]);
    assert!(
        device_enroll.status.success(),
        "device enroll stderr: {}",
        String::from_utf8_lossy(&device_enroll.stderr)
    );
    let enroll_json: serde_json::Value =
        serde_json::from_slice(&device_enroll.stdout).expect("enroll json");
    let device_id = enroll_json
        .get("device")
        .and_then(|v| v.get("device_id"))
        .and_then(|v| v.as_str())
        .expect("device id in enroll output")
        .to_string();

    let verify_chain = run_cli(&[
        "trust",
        "device",
        "verify-chain",
        &vault_path,
        "--device-id",
        &device_id,
        "--now-ms",
        "13",
    ]);
    assert!(
        verify_chain.status.success(),
        "verify-chain stderr: {}",
        String::from_utf8_lossy(&verify_chain.stderr)
    );

    let listed = run_cli(&["trust", "device", "list", &vault_path]);
    assert!(
        listed.status.success(),
        "list stderr: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let list_json: serde_json::Value =
        serde_json::from_slice(&listed.stdout).expect("list json");
    let devices = list_json
        .get("devices")
        .and_then(|v| v.as_array())
        .expect("devices array");
    assert!(!devices.is_empty());
    assert!(devices.iter().any(|d| {
        d.get("device_id")
            .and_then(|v| v.as_str())
            .map(|id| id == device_id)
            .unwrap_or(false)
    }));
}
