use kc_core::hashing::blake3_hex_prefixed;
use kc_core::recovery_escrow::{
    canonical_descriptor_hash, normalize_escrow_descriptors, normalize_provider_configs,
    provider_priority, RecoveryEscrowDescriptorV2, RecoveryEscrowProvider,
    RecoveryEscrowProviderConfigV3, RecoveryEscrowReadRequest, RecoveryEscrowWriteRequest,
    ESCROW_PROVIDER_PRIORITY,
};
use kc_core::recovery_escrow_aws::{AwsRecoveryEscrowConfig, AwsRecoveryEscrowProvider};
use kc_core::recovery_escrow_azure::{AzureRecoveryEscrowConfig, AzureRecoveryEscrowProvider};
use kc_core::recovery_escrow_gcp::{GcpRecoveryEscrowConfig, GcpRecoveryEscrowProvider};
use kc_core::recovery_escrow_hsm::{HsmRecoveryEscrowConfig, HsmRecoveryEscrowProvider};
use kc_core::recovery_escrow_local::LocalRecoveryEscrowProvider;
use kc_core::recovery_escrow_private_kms::{
    PrivateKmsRecoveryEscrowConfig, PrivateKmsRecoveryEscrowProvider,
};

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

#[test]
fn recovery_escrow_gcp_and_azure_report_unavailable_without_emulation() {
    std::env::remove_var("KC_RECOVERY_ESCROW_GCP_EMULATE_DIR");
    std::env::remove_var("KC_RECOVERY_ESCROW_AZURE_EMULATE_DIR");

    let gcp = GcpRecoveryEscrowProvider::new(GcpRecoveryEscrowConfig {
        project_id: "kc-local".to_string(),
        location: "global".to_string(),
        key_ring: "knowledgecore".to_string(),
        key_name: "recovery".to_string(),
        secret_prefix: "kc/recovery".to_string(),
    });
    let azure = AzureRecoveryEscrowProvider::new(AzureRecoveryEscrowConfig {
        key_vault_url: "https://knowledgecore-local.vault.azure.net".to_string(),
        key_name: "recovery".to_string(),
        secret_prefix: "kc/recovery".to_string(),
    });

    assert!(!gcp.status().expect("gcp status").available);
    assert!(!azure.status().expect("azure status").available);

    let gcp_err = gcp
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            key_blob: b"blob",
            now_ms: 1,
        })
        .expect_err("gcp write must fail when unavailable");
    assert_eq!(gcp_err.code, "KC_RECOVERY_ESCROW_UNAVAILABLE");

    let azure_err = azure
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            key_blob: b"blob",
            now_ms: 1,
        })
        .expect_err("azure write must fail when unavailable");
    assert_eq!(azure_err.code, "KC_RECOVERY_ESCROW_UNAVAILABLE");
}

#[test]
fn recovery_escrow_hsm_and_private_kms_report_unavailable_without_emulation() {
    std::env::remove_var("KC_RECOVERY_ESCROW_HSM_EMULATE_DIR");
    std::env::remove_var("KC_RECOVERY_ESCROW_PRIVATE_KMS_EMULATE_DIR");

    let hsm = HsmRecoveryEscrowProvider::new(HsmRecoveryEscrowConfig {
        cluster: "kc-hsm".to_string(),
        key_slot: "slot-0".to_string(),
        secret_prefix: "kc/recovery".to_string(),
    });
    let private_kms = PrivateKmsRecoveryEscrowProvider::new(PrivateKmsRecoveryEscrowConfig {
        endpoint: "https://private-kms.local".to_string(),
        key_alias: "recovery".to_string(),
        tenant: "tenant-a".to_string(),
        secret_prefix: "kc/recovery".to_string(),
    });

    assert!(!hsm.status().expect("hsm status").available);
    assert!(!private_kms.status().expect("private_kms status").available);

    let hsm_err = hsm
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            key_blob: b"blob",
            now_ms: 1,
        })
        .expect_err("hsm write must fail when unavailable");
    assert_eq!(hsm_err.code, "KC_RECOVERY_ESCROW_UNAVAILABLE");

    let private_kms_err = private_kms
        .write(RecoveryEscrowWriteRequest {
            vault_id: "vault-1",
            payload_hash: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            key_blob: b"blob",
            now_ms: 1,
        })
        .expect_err("private_kms write must fail when unavailable");
    assert_eq!(private_kms_err.code, "KC_RECOVERY_ESCROW_UNAVAILABLE");
}

#[test]
fn recovery_escrow_provider_priority_and_ordering_are_deterministic() {
    assert_eq!(
        ESCROW_PROVIDER_PRIORITY,
        ["aws", "gcp", "azure", "hsm", "local", "private_kms"]
    );
    assert!(provider_priority("aws") < provider_priority("gcp"));
    assert!(provider_priority("gcp") < provider_priority("azure"));
    assert!(provider_priority("azure") < provider_priority("hsm"));
    assert!(provider_priority("hsm") < provider_priority("local"));
    assert!(provider_priority("local") < provider_priority("private_kms"));

    let mut providers = vec![
        RecoveryEscrowProviderConfigV3 {
            provider_id: "private_kms".to_string(),
            config_ref: "pk".to_string(),
            enabled: true,
            updated_at_ms: 6,
        },
        RecoveryEscrowProviderConfigV3 {
            provider_id: "local".to_string(),
            config_ref: "z".to_string(),
            enabled: true,
            updated_at_ms: 3,
        },
        RecoveryEscrowProviderConfigV3 {
            provider_id: "azure".to_string(),
            config_ref: "z2".to_string(),
            enabled: true,
            updated_at_ms: 4,
        },
        RecoveryEscrowProviderConfigV3 {
            provider_id: "aws".to_string(),
            config_ref: "a".to_string(),
            enabled: true,
            updated_at_ms: 1,
        },
        RecoveryEscrowProviderConfigV3 {
            provider_id: "gcp".to_string(),
            config_ref: "m".to_string(),
            enabled: true,
            updated_at_ms: 2,
        },
        RecoveryEscrowProviderConfigV3 {
            provider_id: "hsm".to_string(),
            config_ref: "h".to_string(),
            enabled: true,
            updated_at_ms: 5,
        },
    ];
    normalize_provider_configs(&mut providers);
    let provider_ids: Vec<String> = providers.into_iter().map(|p| p.provider_id).collect();
    assert_eq!(
        provider_ids,
        vec!["aws", "gcp", "azure", "hsm", "local", "private_kms"]
    );

    let mut descs = vec![
        RecoveryEscrowDescriptorV2 {
            provider: "private_kms".to_string(),
            provider_ref: "pk".to_string(),
            key_id: "k6".to_string(),
            wrapped_at_ms: 6,
        },
        RecoveryEscrowDescriptorV2 {
            provider: "local".to_string(),
            provider_ref: "c".to_string(),
            key_id: "k3".to_string(),
            wrapped_at_ms: 3,
        },
        RecoveryEscrowDescriptorV2 {
            provider: "azure".to_string(),
            provider_ref: "d".to_string(),
            key_id: "k4".to_string(),
            wrapped_at_ms: 4,
        },
        RecoveryEscrowDescriptorV2 {
            provider: "aws".to_string(),
            provider_ref: "b".to_string(),
            key_id: "k1".to_string(),
            wrapped_at_ms: 1,
        },
        RecoveryEscrowDescriptorV2 {
            provider: "gcp".to_string(),
            provider_ref: "a".to_string(),
            key_id: "k2".to_string(),
            wrapped_at_ms: 2,
        },
        RecoveryEscrowDescriptorV2 {
            provider: "hsm".to_string(),
            provider_ref: "h".to_string(),
            key_id: "k5".to_string(),
            wrapped_at_ms: 5,
        },
    ];
    normalize_escrow_descriptors(&mut descs);
    let ordered: Vec<String> = descs.into_iter().map(|d| d.provider).collect();
    assert_eq!(
        ordered,
        vec!["aws", "gcp", "azure", "hsm", "local", "private_kms"]
    );
}
