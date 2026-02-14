import type {
  DesktopRpcApi,
  LocatorResolveReq,
  LocatorResolveRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function loadDocumentRange(
  api: DesktopRpcApi,
  req: LocatorResolveReq
): Promise<ViewState<LocatorResolveRes>> {
  return nextStateFromRpc(await api.locatorResolve(req));
}
