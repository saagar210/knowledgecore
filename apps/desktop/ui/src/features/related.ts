import type {
  DesktopRpcApi,
  SearchQueryReq,
  SearchQueryRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function loadRelated(
  api: DesktopRpcApi,
  req: SearchQueryReq
): Promise<ViewState<SearchQueryRes>> {
  return nextStateFromRpc(await api.searchQuery(req));
}
