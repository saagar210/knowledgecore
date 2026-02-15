import { describe, expect, it } from "vitest";
import { rpc, rpcMethods } from "../src/api/rpc";

describe("rpc client", () => {
  it("returns not wired error outside tauri runtime", async () => {
    const response = await rpc("vault_open", { vault_path: "/tmp/demo" });
    expect(response.ok).toBe(false);
    if (!response.ok) {
      expect(response.error.code).toBe("KC_RPC_NOT_WIRED");
    }
  });

  it("exposes all v1 method wrappers", () => {
    expect(Object.keys(rpcMethods)).toEqual([
      "vaultInit",
      "vaultOpen",
      "trustIdentityStart",
      "trustIdentityComplete",
      "trustDeviceEnroll",
      "trustDeviceVerifyChain",
      "trustDeviceList",
      "trustProviderAdd",
      "trustProviderDisable",
      "trustProviderList",
      "trustPolicySet",
      "vaultLockStatus",
      "vaultUnlock",
      "vaultLock",
      "vaultEncryptionStatus",
      "vaultEncryptionEnable",
      "vaultEncryptionMigrate",
      "vaultRecoveryStatus",
      "vaultRecoveryEscrowStatus",
      "vaultRecoveryEscrowEnable",
      "vaultRecoveryEscrowRotate",
      "vaultRecoveryEscrowRestore",
      "vaultRecoveryGenerate",
      "vaultRecoveryVerify",
      "ingestScanFolder",
      "ingestInboxStart",
      "ingestInboxStop",
      "searchQuery",
      "locatorResolve",
      "exportBundle",
      "verifyBundle",
      "askQuestion",
      "eventsList",
      "jobsList",
      "syncStatus",
      "syncPush",
      "syncPull",
      "syncMergePreview",
      "lineageQuery",
      "lineageQueryV2",
      "lineageOverlayAdd",
      "lineageOverlayRemove",
      "lineageOverlayList",
      "lineageLockAcquire",
      "lineageLockRelease",
      "lineageLockStatus",
      "lineageRoleGrant",
      "lineageRoleRevoke",
      "lineageRoleList",
      "lineageLockAcquireScope"
    ]);
  });

  it("does not expose retired preview rpc methods", () => {
    expect(Object.keys(rpcMethods)).not.toContain("previewStatus");
    expect(Object.keys(rpcMethods)).not.toContain("previewCapability");
  });
});
