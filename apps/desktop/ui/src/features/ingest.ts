import type {
  DesktopRpcApi,
  IngestInboxStartReq,
  IngestInboxStartRes,
  IngestInboxStopReq,
  IngestInboxStopRes,
  IngestScanFolderReq,
  IngestScanFolderRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function ingestScanFolder(
  api: DesktopRpcApi,
  req: IngestScanFolderReq
): Promise<ViewState<IngestScanFolderRes>> {
  return nextStateFromRpc(await api.ingestScanFolder(req));
}

export async function ingestInboxStart(
  api: DesktopRpcApi,
  req: IngestInboxStartReq
): Promise<ViewState<IngestInboxStartRes>> {
  return nextStateFromRpc(await api.ingestInboxStart(req));
}

export async function ingestInboxStop(
  api: DesktopRpcApi,
  req: IngestInboxStopReq
): Promise<ViewState<IngestInboxStopRes>> {
  return nextStateFromRpc(await api.ingestInboxStop(req));
}
