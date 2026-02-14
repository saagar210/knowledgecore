import type {
  DesktopRpcApi,
  LineageQueryReq,
  LineageQueryRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function queryLineage(
  api: DesktopRpcApi,
  req: LineageQueryReq
): Promise<ViewState<LineageQueryRes>> {
  return nextStateFromRpc(await api.lineageQuery(req));
}
