import type {
  DesktopRpcApi,
  LineageOverlayAddReq,
  LineageOverlayAddRes,
  LineageOverlayListReq,
  LineageOverlayListRes,
  LineageOverlayRemoveReq,
  LineageOverlayRemoveRes,
  LineageQueryReq,
  LineageQueryRes,
  LineageQueryV2Req,
  LineageQueryV2Res
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function queryLineage(
  api: DesktopRpcApi,
  req: LineageQueryReq
): Promise<ViewState<LineageQueryRes>> {
  return nextStateFromRpc(await api.lineageQuery(req));
}

export async function queryLineageV2(
  api: DesktopRpcApi,
  req: LineageQueryV2Req
): Promise<ViewState<LineageQueryV2Res>> {
  return nextStateFromRpc(await api.lineageQueryV2(req));
}

export async function addLineageOverlay(
  api: DesktopRpcApi,
  req: LineageOverlayAddReq
): Promise<ViewState<LineageOverlayAddRes>> {
  return nextStateFromRpc(await api.lineageOverlayAdd(req));
}

export async function removeLineageOverlay(
  api: DesktopRpcApi,
  req: LineageOverlayRemoveReq
): Promise<ViewState<LineageOverlayRemoveRes>> {
  return nextStateFromRpc(await api.lineageOverlayRemove(req));
}

export async function listLineageOverlays(
  api: DesktopRpcApi,
  req: LineageOverlayListReq
): Promise<ViewState<LineageOverlayListRes>> {
  return nextStateFromRpc(await api.lineageOverlayList(req));
}
