import type {
  DesktopRpcApi,
  LineageLockAcquireReq,
  LineageLockAcquireScopeReq,
  LineageLockAcquireScopeRes,
  LineageLockAcquireRes,
  LineageLockReleaseReq,
  LineageLockReleaseRes,
  LineageLockStatusReq,
  LineageLockStatusRes,
  LineageOverlayAddReq,
  LineageOverlayAddRes,
  LineageOverlayListReq,
  LineageOverlayListRes,
  LineageOverlayRemoveReq,
  LineageOverlayRemoveRes,
  LineageQueryReq,
  LineageQueryRes,
  LineageQueryV2Req,
  LineageQueryV2Res,
  LineageRoleGrantReq,
  LineageRoleGrantRes,
  LineageRoleListReq,
  LineageRoleListRes,
  LineageRoleRevokeReq,
  LineageRoleRevokeRes
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

export async function acquireLineageLock(
  api: DesktopRpcApi,
  req: LineageLockAcquireReq
): Promise<ViewState<LineageLockAcquireRes>> {
  return nextStateFromRpc(await api.lineageLockAcquire(req));
}

export async function releaseLineageLock(
  api: DesktopRpcApi,
  req: LineageLockReleaseReq
): Promise<ViewState<LineageLockReleaseRes>> {
  return nextStateFromRpc(await api.lineageLockRelease(req));
}

export async function loadLineageLockStatus(
  api: DesktopRpcApi,
  req: LineageLockStatusReq
): Promise<ViewState<LineageLockStatusRes>> {
  return nextStateFromRpc(await api.lineageLockStatus(req));
}

export async function grantLineageRole(
  api: DesktopRpcApi,
  req: LineageRoleGrantReq
): Promise<ViewState<LineageRoleGrantRes>> {
  return nextStateFromRpc(await api.lineageRoleGrant(req));
}

export async function revokeLineageRole(
  api: DesktopRpcApi,
  req: LineageRoleRevokeReq
): Promise<ViewState<LineageRoleRevokeRes>> {
  return nextStateFromRpc(await api.lineageRoleRevoke(req));
}

export async function listLineageRoles(
  api: DesktopRpcApi,
  req: LineageRoleListReq
): Promise<ViewState<LineageRoleListRes>> {
  return nextStateFromRpc(await api.lineageRoleList(req));
}

export async function acquireLineageScopeLock(
  api: DesktopRpcApi,
  req: LineageLockAcquireScopeReq
): Promise<ViewState<LineageLockAcquireScopeRes>> {
  return nextStateFromRpc(await api.lineageLockAcquireScope(req));
}
