import type {
  DesktopRpcApi,
  JobsListReq,
  JobsListRes,
  VaultEncryptionEnableReq,
  VaultEncryptionEnableRes,
  VaultEncryptionMigrateReq,
  VaultEncryptionMigrateRes,
  VaultEncryptionStatusReq,
  VaultEncryptionStatusRes,
  SyncPullReq,
  SyncPullRes,
  SyncPushReq,
  SyncPushRes,
  SyncStatusReq,
  SyncStatusRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function loadSettingsDependencies(
  api: DesktopRpcApi,
  req: JobsListReq
): Promise<ViewState<JobsListRes>> {
  return nextStateFromRpc(await api.jobsList(req));
}

export async function loadVaultEncryptionStatus(
  api: DesktopRpcApi,
  req: VaultEncryptionStatusReq
): Promise<ViewState<VaultEncryptionStatusRes>> {
  return nextStateFromRpc(await api.vaultEncryptionStatus(req));
}

export async function enableVaultEncryption(
  api: DesktopRpcApi,
  req: VaultEncryptionEnableReq
): Promise<ViewState<VaultEncryptionEnableRes>> {
  return nextStateFromRpc(await api.vaultEncryptionEnable(req));
}

export async function migrateVaultEncryption(
  api: DesktopRpcApi,
  req: VaultEncryptionMigrateReq
): Promise<ViewState<VaultEncryptionMigrateRes>> {
  return nextStateFromRpc(await api.vaultEncryptionMigrate(req));
}

export async function loadSyncStatus(
  api: DesktopRpcApi,
  req: SyncStatusReq
): Promise<ViewState<SyncStatusRes>> {
  return nextStateFromRpc(await api.syncStatus(req));
}

export async function runSyncPush(
  api: DesktopRpcApi,
  req: SyncPushReq
): Promise<ViewState<SyncPushRes>> {
  return nextStateFromRpc(await api.syncPush(req));
}

export async function runSyncPull(
  api: DesktopRpcApi,
  req: SyncPullReq
): Promise<ViewState<SyncPullRes>> {
  return nextStateFromRpc(await api.syncPull(req));
}
