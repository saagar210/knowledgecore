import { createDesktopRpcApi } from "./api/rpc";
import * as askFeature from "./features/ask";
import * as docFeature from "./features/document";
import * as eventsFeature from "./features/events";
import * as exportFeature from "./features/exportVerify";
import * as ingestFeature from "./features/ingest";
import * as lineageFeature from "./features/lineage";
import * as relatedFeature from "./features/related";
import * as searchFeature from "./features/search";
import * as settingsFeature from "./features/settings";
import * as vaultFeature from "./features/vault";
import { createDesktopApp } from "./main";
import type { AppRoute } from "./routes";
import type { ViewState } from "./state/appState";

const app = createDesktopApp();
const api = createDesktopRpcApi();
const outputs: Record<AppRoute, string> = {
  vault: "",
  ingest: "",
  search: "",
  document: "",
  related: "",
  ask: "",
  export: "",
  verify: "",
  events: "",
  settings: "",
  lineage: ""
};

let activeRoute: AppRoute = "vault";

function nowMs(): number {
  return Date.now();
}

function byId<T extends HTMLElement>(id: string): T | null {
  return document.getElementById(id) as T | null;
}

function inputValue(id: string, fallback = ""): string {
  const value = byId<HTMLInputElement>(id)?.value?.trim();
  return value && value.length > 0 ? value : fallback;
}

function optionalNumber(id: string): number | undefined {
  const raw = byId<HTMLInputElement>(id)?.value?.trim();
  if (!raw) return undefined;
  const parsed = Number.parseInt(raw, 10);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function numberValue(id: string, fallback: number): number {
  const parsed = optionalNumber(id);
  return typeof parsed === "number" ? parsed : fallback;
}

function checked(id: string): boolean {
  return Boolean(byId<HTMLInputElement>(id)?.checked);
}

function vaultPath(): string {
  return inputValue("vault-path", "/tmp/knowledgecore-vault");
}

function renderStatusBadges(): void {
  app.routeDescriptors.forEach((route) => {
    const badge = document.querySelector<HTMLElement>(`[data-badge='${route.id}']`);
    if (!badge) return;
    const state = app.featureStates[route.id].status;
    badge.textContent = state;
    badge.dataset.state = state;
  });
}

function setOutput(route: AppRoute): void {
  const out = byId<HTMLPreElement>("result-output");
  if (!out) return;
  out.textContent = outputs[route] || `No output yet for ${route}.`;
}

function showRoute(route: AppRoute): void {
  activeRoute = route;
  document.querySelectorAll<HTMLElement>("[data-route]").forEach((button) => {
    button.dataset.active = button.dataset.route === route ? "true" : "false";
  });
  document.querySelectorAll<HTMLElement>("[data-panel]").forEach((panel) => {
    panel.hidden = panel.dataset.panel !== route;
  });
  setOutput(route);
}

function encodeState(state: ViewState<unknown>): string {
  if (state.kind === "data") {
    return JSON.stringify({ kind: "data", value: state.value }, null, 2);
  }
  if (state.kind === "error") {
    return JSON.stringify(
      {
        kind: "error",
        route: state.route,
        code: state.code
      },
      null,
      2
    );
  }
  return JSON.stringify({ kind: "idle" }, null, 2);
}

async function execute(route: AppRoute, action: () => Promise<ViewState<unknown>>): Promise<void> {
  app.featureStates[route].status = "loading";
  renderStatusBadges();
  try {
    const state = await action();
    if (state.kind === "data") {
      app.featureStates[route].status = "ready";
      delete app.featureStates[route].lastErrorCode;
    } else if (state.kind === "error") {
      app.featureStates[route].status = "error";
      app.featureStates[route].lastErrorCode = state.code;
    } else {
      app.featureStates[route].status = "idle";
      delete app.featureStates[route].lastErrorCode;
    }
    outputs[route] = encodeState(state);
  } catch (_error) {
    app.featureStates[route].status = "error";
    app.featureStates[route].lastErrorCode = "KC_UI_ACTION_FAILED";
    outputs[route] = JSON.stringify(
      {
        kind: "error",
        route: "fatal-error",
        code: "KC_UI_ACTION_FAILED"
      },
      null,
      2
    );
  }
  renderStatusBadges();
  if (activeRoute === route) setOutput(route);
}

function routePanel(route: AppRoute): string {
  switch (route) {
    case "vault":
      return `
        <h2>Vault</h2>
        <div class="field-grid">
          <label>Vault slug<input id="vault-slug" value="default-vault" /></label>
        </div>
        <div class="actions">
          <button data-action="vault-open">Open Vault</button>
          <button data-action="vault-init">Init Vault</button>
        </div>
      `;
    case "ingest":
      return `
        <h2>Ingest</h2>
        <div class="field-grid">
          <label>Scan root<input id="ingest-scan-root" value="/tmp" /></label>
          <label>Source kind<input id="ingest-source-kind" value="fs" /></label>
          <label>File path<input id="ingest-file-path" value="/tmp/example.txt" /></label>
          <label>Job ID<input id="ingest-job-id" value="" /></label>
        </div>
        <div class="actions">
          <button data-action="ingest-scan">Scan Folder</button>
          <button data-action="ingest-start">Start Inbox</button>
          <button data-action="ingest-stop">Stop Inbox</button>
        </div>
      `;
    case "search":
      return `
        <h2>Search</h2>
        <div class="field-grid">
          <label>Query<input id="search-query" value="knowledge" /></label>
          <label>Limit<input id="search-limit" value="10" /></label>
        </div>
        <div class="actions">
          <button data-action="search-run">Run Search</button>
        </div>
      `;
    case "document":
      return `
        <h2>Document</h2>
        <div class="field-grid">
          <label>Doc ID<input id="doc-id" value="" /></label>
          <label>Canonical hash<input id="doc-hash" value="" /></label>
          <label>Range start<input id="doc-range-start" value="0" /></label>
          <label>Range end<input id="doc-range-end" value="120" /></label>
        </div>
        <div class="actions">
          <button data-action="document-resolve">Resolve Locator</button>
        </div>
      `;
    case "related":
      return `
        <h2>Related</h2>
        <div class="field-grid">
          <label>Query<input id="related-query" value="related topic" /></label>
          <label>Limit<input id="related-limit" value="10" /></label>
        </div>
        <div class="actions">
          <button data-action="related-run">Load Related</button>
        </div>
      `;
    case "ask":
      return `
        <h2>Ask</h2>
        <div class="field-grid">
          <label>Question<input id="ask-question" value="What changed in the latest run?" /></label>
        </div>
        <div class="actions">
          <button data-action="ask-run">Ask Question</button>
        </div>
      `;
    case "export":
      return `
        <h2>Export</h2>
        <div class="field-grid">
          <label>Export dir<input id="export-dir" value="/tmp" /></label>
          <label class="checkbox"><input id="export-vectors" type="checkbox" /> Include vectors</label>
        </div>
        <div class="actions">
          <button data-action="export-run">Export Bundle</button>
        </div>
      `;
    case "verify":
      return `
        <h2>Verify</h2>
        <div class="field-grid">
          <label>Bundle path<input id="verify-path" value="" /></label>
        </div>
        <div class="actions">
          <button data-action="verify-run">Verify Bundle</button>
        </div>
      `;
    case "events":
      return `
        <h2>Events</h2>
        <div class="field-grid">
          <label>Limit<input id="events-limit" value="20" /></label>
        </div>
        <div class="actions">
          <button data-action="events-list">List Events</button>
          <button data-action="jobs-list">List Jobs</button>
        </div>
      `;
    case "settings":
      return `
        <h2>Settings</h2>
        <div class="field-grid">
          <label>Provider<input id="settings-provider" value="default" /></label>
          <label>Issuer<input id="settings-issuer" value="https://issuer.example" /></label>
          <label>Vault passphrase<input id="settings-passphrase" type="password" value="" /></label>
        </div>
        <div class="actions">
          <button data-action="settings-deps">Load Dependencies</button>
          <button data-action="settings-lock-status">Lock Status</button>
          <button data-action="settings-lock">Lock Vault</button>
          <button data-action="settings-unlock">Unlock Vault</button>
          <button data-action="settings-encryption-status">Encryption Status</button>
          <button data-action="settings-recovery-status">Recovery Status</button>
          <button data-action="settings-escrow-status">Escrow Status</button>
          <button data-action="settings-trust-list">List Trust Providers</button>
          <button data-action="settings-trust-discover">Discover Trust Provider</button>
        </div>
      `;
    case "lineage":
      return `
        <h2>Lineage</h2>
        <div class="field-grid">
          <label>Seed doc ID<input id="lineage-seed-doc" value="" /></label>
          <label>Depth<input id="lineage-depth" value="2" /></label>
          <label>Doc ID (lock/overlays)<input id="lineage-doc-id" value="" /></label>
          <label>Owner<input id="lineage-owner" value="desktop-user" /></label>
          <label>Token (release)<input id="lineage-token" value="" /></label>
        </div>
        <div class="actions">
          <button data-action="lineage-query">Query Lineage v2</button>
          <button data-action="lineage-lock-status">Lock Status</button>
          <button data-action="lineage-lock-acquire">Acquire Lock</button>
          <button data-action="lineage-lock-release">Release Lock</button>
          <button data-action="lineage-overlays-list">List Overlays</button>
          <button data-action="lineage-roles-list">List Roles</button>
          <button data-action="lineage-policies-list">List Policies</button>
        </div>
      `;
    default:
      return "<h2>Unknown route</h2>";
  }
}

function renderShell(): void {
  const mount = byId<HTMLElement>("app");
  if (!mount) return;
  const nav = app.routeDescriptors
    .map(
      (route) =>
        `<button class="nav-btn" data-route="${route.id}" data-active="false">
          <span>${route.title}</span>
          <small data-badge="${route.id}" data-state="idle">idle</small>
        </button>`
    )
    .join("");
  const panels = app.routeDescriptors
    .map(
      (route) =>
        `<section class="panel" data-panel="${route.id}" hidden>${routePanel(route.id)}</section>`
    )
    .join("");

  mount.innerHTML = `
    <style>
      :root { color-scheme: light; }
      body { margin: 0; background: linear-gradient(180deg, #eef4ff 0%, #f8fbff 100%); font-family: "SF Pro Text", "Segoe UI", sans-serif; color: #17233a; }
      .layout { display: grid; grid-template-columns: 260px 1fr; min-height: 100vh; }
      .sidebar { background: #11213d; color: #f0f6ff; padding: 16px; }
      .brand { font-size: 20px; font-weight: 650; margin-bottom: 14px; }
      .vault { font-size: 12px; margin-bottom: 14px; }
      .vault input { width: 100%; margin-top: 6px; padding: 8px; border-radius: 8px; border: 1px solid #2a3f63; background: #182c4d; color: #f7fbff; }
      .nav { display: grid; gap: 8px; }
      .nav-btn { display: grid; grid-template-columns: 1fr auto; align-items: center; width: 100%; border: 0; border-radius: 10px; padding: 10px 12px; text-align: left; cursor: pointer; background: #1b3156; color: #e8f1ff; }
      .nav-btn[data-active="true"] { background: #3a6cf4; color: #fff; }
      .nav-btn small { border-radius: 999px; padding: 2px 8px; background: #27416e; font-size: 11px; text-transform: uppercase; }
      .nav-btn small[data-state="ready"] { background: #1f6f43; }
      .nav-btn small[data-state="error"] { background: #8c2130; }
      .content { padding: 20px; }
      .panel { background: #ffffff; border: 1px solid #d8e3f5; border-radius: 12px; padding: 16px; box-shadow: 0 8px 24px rgba(17, 33, 61, 0.07); }
      .panel h2 { margin: 0 0 12px; }
      .field-grid { display: grid; grid-template-columns: repeat(2, minmax(220px, 1fr)); gap: 10px; }
      .field-grid label { display: grid; gap: 6px; font-size: 13px; color: #314769; }
      .field-grid input { border: 1px solid #c9d7ee; border-radius: 8px; padding: 8px 10px; font-size: 14px; }
      .field-grid .checkbox { grid-template-columns: auto 1fr; align-items: center; }
      .actions { margin-top: 14px; display: flex; flex-wrap: wrap; gap: 8px; }
      .actions button { border: 0; border-radius: 8px; padding: 9px 12px; background: #2f6ef2; color: #fff; cursor: pointer; }
      .result { margin-top: 14px; background: #0f1728; color: #dbe8ff; border-radius: 10px; padding: 12px; overflow: auto; max-height: 320px; }
      .result pre { margin: 0; font-family: "SF Mono", Menlo, monospace; font-size: 12px; white-space: pre-wrap; word-break: break-word; }
      @media (max-width: 980px) {
        .layout { grid-template-columns: 1fr; }
        .sidebar { position: sticky; top: 0; z-index: 2; }
        .field-grid { grid-template-columns: 1fr; }
      }
    </style>
    <div class="layout">
      <aside class="sidebar">
        <div class="brand">${app.name}</div>
        <label class="vault">Vault path
          <input id="vault-path" value="/tmp/knowledgecore-vault" />
        </label>
        <nav class="nav">${nav}</nav>
      </aside>
      <main class="content">
        ${panels}
        <div class="result"><pre id="result-output">No output yet.</pre></div>
      </main>
    </div>
  `;
}

async function handleAction(action: string): Promise<void> {
  switch (action) {
    case "vault-open":
      await execute("vault", () => vaultFeature.vaultOpen(api, { vault_path: vaultPath() }));
      return;
    case "vault-init":
      await execute("vault", () =>
        vaultFeature.vaultInit(api, {
          vault_path: vaultPath(),
          vault_slug: inputValue("vault-slug", "default-vault"),
          now_ms: nowMs()
        })
      );
      return;
    case "ingest-scan":
      await execute("ingest", () =>
        ingestFeature.ingestScanFolder(api, {
          vault_path: vaultPath(),
          scan_root: inputValue("ingest-scan-root", "/tmp"),
          source_kind: inputValue("ingest-source-kind", "fs"),
          now_ms: nowMs()
        })
      );
      return;
    case "ingest-start":
      await execute("ingest", () =>
        ingestFeature.ingestInboxStart(api, {
          vault_path: vaultPath(),
          file_path: inputValue("ingest-file-path", "/tmp/example.txt"),
          source_kind: inputValue("ingest-source-kind", "fs"),
          now_ms: nowMs()
        })
      );
      return;
    case "ingest-stop":
      await execute("ingest", () =>
        ingestFeature.ingestInboxStop(api, {
          vault_path: vaultPath(),
          job_id: inputValue("ingest-job-id")
        })
      );
      return;
    case "search-run":
      await execute("search", () =>
        searchFeature.runSearch(api, {
          vault_path: vaultPath(),
          query: inputValue("search-query", "knowledge"),
          limit: optionalNumber("search-limit"),
          now_ms: nowMs()
        })
      );
      return;
    case "document-resolve":
      await execute("document", () =>
        docFeature.loadDocumentRange(api, {
          vault_path: vaultPath(),
          locator: {
            v: 1,
            doc_id: inputValue("doc-id"),
            canonical_hash: inputValue("doc-hash"),
            range: {
              start: numberValue("doc-range-start", 0),
              end: numberValue("doc-range-end", 120)
            }
          }
        })
      );
      return;
    case "related-run":
      await execute("related", () =>
        relatedFeature.loadRelated(api, {
          vault_path: vaultPath(),
          query: inputValue("related-query", "related topic"),
          limit: optionalNumber("related-limit"),
          now_ms: nowMs()
        })
      );
      return;
    case "ask-run":
      await execute("ask", () =>
        askFeature.askQuestion(api, {
          vault_path: vaultPath(),
          question: inputValue("ask-question"),
          now_ms: nowMs()
        })
      );
      return;
    case "export-run":
      await execute("export", () =>
        exportFeature.exportBundle(api, {
          vault_path: vaultPath(),
          export_dir: inputValue("export-dir", "/tmp"),
          include_vectors: checked("export-vectors"),
          now_ms: nowMs()
        })
      );
      return;
    case "verify-run":
      await execute("verify", () =>
        exportFeature.verifyBundle(api, { bundle_path: inputValue("verify-path") })
      );
      return;
    case "events-list":
      await execute("events", () =>
        eventsFeature.listEvents(api, {
          vault_path: vaultPath(),
          limit: optionalNumber("events-limit")
        })
      );
      return;
    case "jobs-list":
      await execute("events", () => eventsFeature.listJobs(api, { vault_path: vaultPath() }));
      return;
    case "settings-deps":
      await execute("settings", () =>
        settingsFeature.loadSettingsDependencies(api, { vault_path: vaultPath() })
      );
      return;
    case "settings-lock-status":
      await execute("settings", () =>
        settingsFeature.loadVaultLockStatus(api, { vault_path: vaultPath() })
      );
      return;
    case "settings-lock":
      await execute("settings", () => settingsFeature.lockVault(api, { vault_path: vaultPath() }));
      return;
    case "settings-unlock":
      await execute("settings", () =>
        settingsFeature.unlockVault(api, {
          vault_path: vaultPath(),
          passphrase: inputValue("settings-passphrase")
        })
      );
      return;
    case "settings-encryption-status":
      await execute("settings", () =>
        settingsFeature.loadVaultEncryptionStatus(api, { vault_path: vaultPath() })
      );
      return;
    case "settings-recovery-status":
      await execute("settings", () =>
        settingsFeature.loadVaultRecoveryStatus(api, { vault_path: vaultPath() })
      );
      return;
    case "settings-escrow-status":
      await execute("settings", () =>
        settingsFeature.loadVaultRecoveryEscrowStatus(api, { vault_path: vaultPath() })
      );
      return;
    case "settings-trust-list":
      await execute("settings", () =>
        settingsFeature.listTrustProviders(api, { vault_path: vaultPath() })
      );
      return;
    case "settings-trust-discover":
      await execute("settings", () =>
        settingsFeature.discoverTrustProvider(api, {
          vault_path: vaultPath(),
          issuer: inputValue("settings-issuer"),
          now_ms: nowMs()
        })
      );
      return;
    case "lineage-query":
      await execute("lineage", () =>
        lineageFeature.queryLineageV2(api, {
          vault_path: vaultPath(),
          seed_doc_id: inputValue("lineage-seed-doc"),
          depth: numberValue("lineage-depth", 2),
          now_ms: nowMs()
        })
      );
      return;
    case "lineage-lock-status":
      await execute("lineage", () =>
        lineageFeature.loadLineageLockStatus(api, {
          vault_path: vaultPath(),
          doc_id: inputValue("lineage-doc-id"),
          now_ms: nowMs()
        })
      );
      return;
    case "lineage-lock-acquire":
      await execute("lineage", () =>
        lineageFeature.acquireLineageLock(api, {
          vault_path: vaultPath(),
          doc_id: inputValue("lineage-doc-id"),
          owner: inputValue("lineage-owner", "desktop-user"),
          now_ms: nowMs()
        })
      );
      return;
    case "lineage-lock-release":
      await execute("lineage", () =>
        lineageFeature.releaseLineageLock(api, {
          vault_path: vaultPath(),
          doc_id: inputValue("lineage-doc-id"),
          token: inputValue("lineage-token")
        })
      );
      return;
    case "lineage-overlays-list":
      await execute("lineage", () =>
        lineageFeature.listLineageOverlays(api, {
          vault_path: vaultPath(),
          doc_id: inputValue("lineage-doc-id")
        })
      );
      return;
    case "lineage-roles-list":
      await execute("lineage", () =>
        lineageFeature.listLineageRoles(api, { vault_path: vaultPath() })
      );
      return;
    case "lineage-policies-list":
      await execute("lineage", () =>
        lineageFeature.listLineagePolicies(api, { vault_path: vaultPath() })
      );
      return;
    default:
      return;
  }
}

function wireEvents(): void {
  document.addEventListener("click", (event) => {
    const target = event.target as HTMLElement | null;
    if (!target) return;
    const routeButton = target.closest<HTMLElement>("[data-route]");
    if (routeButton?.dataset.route) {
      showRoute(routeButton.dataset.route as AppRoute);
      return;
    }
    const actionButton = target.closest<HTMLElement>("[data-action]");
    if (actionButton?.dataset.action) {
      void handleAction(actionButton.dataset.action);
    }
  });
}

renderShell();
renderStatusBadges();
showRoute(activeRoute);
wireEvents();
