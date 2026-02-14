import { describe, expect, it } from "vitest";
import type { DesktopRpcApi, RpcResp } from "../src/api/rpc";
import { askQuestion } from "../src/features/ask";
import { loadDocumentRange } from "../src/features/document";
import { listEvents, listJobs } from "../src/features/events";
import { exportBundle, verifyBundle } from "../src/features/exportVerify";
import {
  ingestInboxStart,
  ingestInboxStop,
  ingestScanFolder
} from "../src/features/ingest";
import { loadRelated } from "../src/features/related";
import { runSearch } from "../src/features/search";
import {
  enableVaultEncryption,
  loadSettingsDependencies,
  loadVaultEncryptionStatus,
  migrateVaultEncryption
} from "../src/features/settings";
import { vaultInit, vaultOpen } from "../src/features/vault";

function ok<T>(data: T): Promise<RpcResp<T>> {
  return Promise.resolve({ ok: true, data });
}

function mockApi(): DesktopRpcApi {
  return {
    vaultInit: () => ok({ vault_id: "v1" }),
    vaultOpen: () => ok({ vault_id: "v1", vault_slug: "demo" }),
    vaultEncryptionStatus: () =>
      ok({
        enabled: false,
        mode: "object_store_xchacha20poly1305",
        key_reference: null,
        kdf_algorithm: "argon2id",
        objects_total: 1,
        objects_encrypted: 0
      }),
    vaultEncryptionEnable: () =>
      ok({
        status: {
          enabled: true,
          mode: "object_store_xchacha20poly1305",
          key_reference: "vault:v1",
          kdf_algorithm: "argon2id",
          objects_total: 1,
          objects_encrypted: 0
        }
      }),
    vaultEncryptionMigrate: () =>
      ok({
        status: {
          enabled: true,
          mode: "object_store_xchacha20poly1305",
          key_reference: "vault:v1",
          kdf_algorithm: "argon2id",
          objects_total: 1,
          objects_encrypted: 1
        },
        migrated_objects: 1,
        already_encrypted_objects: 0,
        event_id: 42
      }),
    ingestScanFolder: () => ok({ ingested: 2 }),
    ingestInboxStart: () => ok({ job_id: "j1", doc_id: "d1" }),
    ingestInboxStop: () => ok({ stopped: true }),
    searchQuery: () => ok({ hits: [{ doc_id: "d1", score: 1, snippet: "s" }] }),
    locatorResolve: () => ok({ text: "doc text" }),
    exportBundle: () => ok({ bundle_path: "/tmp/bundle" }),
    verifyBundle: () => ok({ exit_code: 0, report: {} }),
    askQuestion: () => ok({ answer_text: "a", trace_path: "/tmp/trace" }),
    eventsList: () => ok({ events: [{ event_id: 1, ts_ms: 1, event_type: "ingest" }] }),
    jobsList: () => ok({ jobs: ["j1"] })
  };
}

describe("feature controllers", () => {
  it("routes all feature actions through rpc envelopes", async () => {
    const api = mockApi();

    expect(
      await vaultInit(api, { vault_path: "/tmp/v", vault_slug: "demo", now_ms: 1 })
    ).toMatchObject({ kind: "data" });
    expect(await vaultOpen(api, { vault_path: "/tmp/v" })).toMatchObject({
      kind: "data"
    });
    expect(
      await ingestScanFolder(api, {
        vault_path: "/tmp/v",
        scan_root: "/tmp/scan",
        source_kind: "notes",
        now_ms: 2
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await ingestInboxStart(api, {
        vault_path: "/tmp/v",
        file_path: "/tmp/f.txt",
        source_kind: "notes",
        now_ms: 2
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await ingestInboxStop(api, {
        vault_path: "/tmp/v",
        job_id: "j1"
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await runSearch(api, {
        vault_path: "/tmp/v",
        query: "q",
        now_ms: 3
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await loadRelated(api, {
        vault_path: "/tmp/v",
        query: "q",
        now_ms: 3
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await loadDocumentRange(api, {
        vault_path: "/tmp/v",
        locator: {
          v: 1,
          doc_id: "d1",
          canonical_hash: "h1",
          range: { start: 0, end: 3 }
        }
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await askQuestion(api, {
        vault_path: "/tmp/v",
        question: "what?",
        now_ms: 4
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await exportBundle(api, {
        vault_path: "/tmp/v",
        export_dir: "/tmp/e",
        include_vectors: true,
        now_ms: 5
      })
    ).toMatchObject({ kind: "data" });
    expect(await verifyBundle(api, { bundle_path: "/tmp/bundle" })).toMatchObject({
      kind: "data"
    });
    expect(await listEvents(api, { vault_path: "/tmp/v" })).toMatchObject({
      kind: "data"
    });
    expect(await listJobs(api, { vault_path: "/tmp/v" })).toMatchObject({
      kind: "data"
    });
    expect(
      await loadSettingsDependencies(api, {
        vault_path: "/tmp/v"
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await loadVaultEncryptionStatus(api, {
        vault_path: "/tmp/v"
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await enableVaultEncryption(api, {
        vault_path: "/tmp/v",
        passphrase: "pass"
      })
    ).toMatchObject({ kind: "data" });
    expect(
      await migrateVaultEncryption(api, {
        vault_path: "/tmp/v",
        passphrase: "pass",
        now_ms: 6
      })
    ).toMatchObject({ kind: "data" });
  });
});
