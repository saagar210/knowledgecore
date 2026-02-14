import type {
  DesktopRpcApi,
  JobsListReq,
  JobsListRes,
  VaultEncryptionEnableReq,
  VaultEncryptionEnableRes,
  VaultEncryptionMigrateReq,
  VaultEncryptionMigrateRes,
  VaultEncryptionStatusReq,
  VaultEncryptionStatusRes
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
