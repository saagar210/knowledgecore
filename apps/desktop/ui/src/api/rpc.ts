export type AppError = {
  schema_version: number;
  code: string;
  category: string;
  message: string;
  retryable: boolean;
  details: unknown;
};

export type RpcOk<T> = { ok: true; data: T };
export type RpcErr = { ok: false; error: AppError };
export type RpcResp<T> = RpcOk<T> | RpcErr;

type TauriInvoke = (cmd: string, args?: unknown) => Promise<unknown>;
type PreviewGlobal = typeof globalThis & { __KC_PHASE_L_PREVIEW__?: boolean };

function notWired(): RpcErr {
  return {
    ok: false,
    error: {
      schema_version: 1,
      code: "KC_RPC_NOT_WIRED",
      category: "rpc",
      message: "RPC invoke layer is not wired in this runtime",
      retryable: true,
      details: {}
    }
  };
}

function tauriInvoke(): TauriInvoke | null {
  const g = globalThis as Record<string, unknown>;
  const tauri = g.__TAURI__ as
    | { core?: { invoke?: TauriInvoke }; tauri?: { invoke?: TauriInvoke } }
    | undefined;
  return tauri?.core?.invoke ?? tauri?.tauri?.invoke ?? null;
}

export async function rpc<TReq, TRes>(cmd: string, req: TReq): Promise<RpcResp<TRes>> {
  const invoke = tauriInvoke();
  if (!invoke) {
    return notWired();
  }

  try {
    const raw = await invoke(cmd, req as unknown);
    if (typeof raw !== "object" || raw === null || !("ok" in raw)) {
      return {
        ok: false,
        error: {
          schema_version: 1,
          code: "KC_RPC_INVALID_RESPONSE",
          category: "rpc",
          message: "RPC response is not a valid envelope",
          retryable: false,
          details: { cmd }
        }
      };
    }
    const envelope = raw as RpcResp<TRes>;
    return envelope;
  } catch (error) {
    return {
      ok: false,
      error: {
        schema_version: 1,
        code: "KC_RPC_INVOKE_FAILED",
        category: "rpc",
        message: "RPC invoke failed",
        retryable: true,
        details: { cmd, error: String(error) }
      }
    };
  }
}

export type VaultInitReq = { vault_path: string; vault_slug: string };
export type VaultInitReqV1 = { vault_path: string; vault_slug: string; now_ms: number };
export type VaultInitRes = { vault_id: string };
export type VaultOpenReq = { vault_path: string };
export type VaultOpenRes = { vault_id: string; vault_slug: string };
export type IngestScanFolderReq = {
  vault_path: string;
  scan_root: string;
  source_kind: string;
  now_ms: number;
};
export type IngestScanFolderRes = { ingested: number };
export type IngestInboxStartReq = {
  vault_path: string;
  file_path: string;
  source_kind: string;
  now_ms: number;
};
export type IngestInboxStartRes = { job_id: string; doc_id: string };
export type IngestInboxStopReq = { vault_path: string; job_id: string };
export type IngestInboxStopRes = { stopped: boolean };
export type SearchQueryReq = { vault_path: string; query: string; now_ms: number; limit?: number };
export type SearchHit = { doc_id: string; score: number; snippet: string };
export type SearchQueryRes = { hits: SearchHit[] };
export type LocatorV1 = { v: number; doc_id: { 0: string } | string; canonical_hash: { 0: string } | string; range: { start: number; end: number }; hints?: unknown };
export type LocatorResolveReq = { vault_path: string; locator: LocatorV1 };
export type LocatorResolveRes = { text: string };
export type ExportBundleReq = {
  vault_path: string;
  export_dir: string;
  include_vectors: boolean;
  now_ms: number;
};
export type ExportBundleRes = { bundle_path: string };
export type VerifyBundleReq = { bundle_path: string };
export type VerifyBundleRes = { exit_code: number; report: unknown };
export type AskQuestionReq = { vault_path: string; question: string; now_ms: number };
export type AskQuestionRes = { answer_text: string; trace_path: string };
export type EventsListReq = { vault_path: string; limit?: number };
export type EventItem = { event_id: number; ts_ms: number; event_type: string };
export type EventsListRes = { events: EventItem[] };
export type JobsListReq = { vault_path: string };
export type JobsListRes = { jobs: string[] };
export type PreviewStatusReq = Record<string, never>;
export type PreviewCapabilityDraft = {
  schema_version: number;
  status: string;
  capability: string;
  activation_phase: string;
  spec_path: string;
  preview_error_code: string;
};
export type PreviewStatusRes = {
  schema_version: number;
  status: string;
  capabilities: PreviewCapabilityDraft[];
};
export type PreviewCapabilityReq = { name: string };
export type PreviewCapabilityRes = { capability: string; status: string };

export const rpcMethods = {
  vaultInit: (req: VaultInitReqV1) => rpc<VaultInitReqV1, VaultInitRes>("vault_init", req),
  vaultOpen: (req: VaultOpenReq) => rpc<VaultOpenReq, VaultOpenRes>("vault_open", req),
  ingestScanFolder: (req: IngestScanFolderReq) => rpc<IngestScanFolderReq, IngestScanFolderRes>("ingest_scan_folder", req),
  ingestInboxStart: (req: IngestInboxStartReq) => rpc<IngestInboxStartReq, IngestInboxStartRes>("ingest_inbox_start", req),
  ingestInboxStop: (req: IngestInboxStopReq) => rpc<IngestInboxStopReq, IngestInboxStopRes>("ingest_inbox_stop", req),
  searchQuery: (req: SearchQueryReq) => rpc<SearchQueryReq, SearchQueryRes>("search_query", req),
  locatorResolve: (req: LocatorResolveReq) => rpc<LocatorResolveReq, LocatorResolveRes>("locator_resolve", req),
  exportBundle: (req: ExportBundleReq) => rpc<ExportBundleReq, ExportBundleRes>("export_bundle", req),
  verifyBundle: (req: VerifyBundleReq) => rpc<VerifyBundleReq, VerifyBundleRes>("verify_bundle", req),
  askQuestion: (req: AskQuestionReq) => rpc<AskQuestionReq, AskQuestionRes>("ask_question", req),
  eventsList: (req: EventsListReq) => rpc<EventsListReq, EventsListRes>("events_list", req),
  jobsList: (req: JobsListReq) => rpc<JobsListReq, JobsListRes>("jobs_list", req)
};

export function previewRpcEnabled(): boolean {
  return (globalThis as PreviewGlobal).__KC_PHASE_L_PREVIEW__ === true;
}

export function createPreviewRpcApi():
  | {
      previewStatus: (req: PreviewStatusReq) => Promise<RpcResp<PreviewStatusRes>>;
      previewCapability: (req: PreviewCapabilityReq) => Promise<RpcResp<PreviewCapabilityRes>>;
    }
  | null {
  if (!previewRpcEnabled()) {
    return null;
  }
  return {
    previewStatus: (req: PreviewStatusReq) => rpc<PreviewStatusReq, PreviewStatusRes>("preview_status", req),
    previewCapability: (req: PreviewCapabilityReq) =>
      rpc<PreviewCapabilityReq, PreviewCapabilityRes>("preview_capability", req)
  };
}

export type DesktopRpcApi = typeof rpcMethods;

export function createDesktopRpcApi(): DesktopRpcApi {
  return rpcMethods;
}
