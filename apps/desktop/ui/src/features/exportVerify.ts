import type {
  DesktopRpcApi,
  ExportBundleReq,
  ExportBundleRes,
  VerifyBundleReq,
  VerifyBundleRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function exportBundle(
  api: DesktopRpcApi,
  req: ExportBundleReq
): Promise<ViewState<ExportBundleRes>> {
  return nextStateFromRpc(await api.exportBundle(req));
}

export async function verifyBundle(
  api: DesktopRpcApi,
  req: VerifyBundleReq
): Promise<ViewState<VerifyBundleRes>> {
  return nextStateFromRpc(await api.verifyBundle(req));
}
