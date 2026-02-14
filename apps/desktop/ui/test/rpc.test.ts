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
      "vaultLockStatus",
      "vaultUnlock",
      "vaultLock",
      "vaultEncryptionStatus",
      "vaultEncryptionEnable",
      "vaultEncryptionMigrate",
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
      "lineageQuery",
      "lineageQueryV2",
      "lineageOverlayAdd",
      "lineageOverlayRemove",
      "lineageOverlayList"
    ]);
  });

  it("does not expose retired preview rpc methods", () => {
    expect(Object.keys(rpcMethods)).not.toContain("previewStatus");
    expect(Object.keys(rpcMethods)).not.toContain("previewCapability");
  });
});
