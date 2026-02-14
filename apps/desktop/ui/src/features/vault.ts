import type {
  DesktopRpcApi,
  VaultInitReqV1,
  VaultInitRes,
  VaultOpenReq,
  VaultOpenRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function vaultInit(
  api: DesktopRpcApi,
  req: VaultInitReqV1
): Promise<ViewState<VaultInitRes>> {
  return nextStateFromRpc(await api.vaultInit(req));
}

export async function vaultOpen(
  api: DesktopRpcApi,
  req: VaultOpenReq
): Promise<ViewState<VaultOpenRes>> {
  return nextStateFromRpc(await api.vaultOpen(req));
}
