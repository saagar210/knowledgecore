use kc_core::hashing::blake3_hex_prefixed;
use kc_core::recovery_escrow::{
    canonical_descriptor_hash, RecoveryEscrowProvider, RecoveryEscrowReadRequest,
    RecoveryEscrowWriteRequest,
};
use kc_core::recovery_escrow_aws::{AwsRecoveryEscrowConfig, AwsRecoveryEscrowProvider};
use kc_core::recovery_escrow_local::LocalRecoveryEscrowProvider;

#[test]
fn recovery_escrow_local_round_trip_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let provider = LocalRecoveryEscrowProvider::new(temp.path().join("escrow"));
    let payload = b"encrypted-key-blob";
    let payload_hash = blake3_hex_prefixed(payload);

    let first = provider
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: &payload_hash,
            key_blob: payload,
            now_ms: 100,
        })
        .expect("write escrow payload");
    let second = provider
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: &payload_hash,
            key_blob: payload,
            now_ms: 100,
        })
        .expect("write escrow payload again");

    assert_eq!(first, second);
    assert_eq!(
        canonical_descriptor_hash(&first).expect("hash descriptor"),
        canonical_descriptor_hash(&second).expect("hash descriptor")
    );

    let loaded = provider
        .read(RecoveryEscrowReadRequest {
            descriptor: &first,
            expected_payload_hash: &payload_hash,
        })
        .expect("read escrow payload");
    assert_eq!(loaded, payload);
}

#[test]
fn recovery_escrow_aws_returns_unavailable_without_emulation() {
    std::env::remove_var("KC_RECOVERY_ESCROW_AWS_EMULATE_DIR");
    let provider = AwsRecoveryEscrowProvider::new(AwsRecoveryEscrowConfig {
        region: "us-east-1".to_string(),
        kms_key_id: "alias/kc-test".to_string(),
        secret_prefix: "kc/recovery".to_string(),
    });

    let status = provider.status().expect("provider status");
    assert!(status.configured);
    assert!(!status.available);

    let err = provider
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            key_blob: b"blob",
            now_ms: 1,
        })
        .expect_err("write must fail when unavailable");
    assert_eq!(err.code, "KC_RECOVERY_ESCROW_UNAVAILABLE");
}
