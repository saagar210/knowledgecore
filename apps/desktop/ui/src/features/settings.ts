import type {
  DesktopRpcApi,
  JobsListReq,
  JobsListRes,
  TrustIdentityStartReq,
  TrustIdentityStartRes,
  TrustIdentityCompleteReq,
  TrustIdentityCompleteRes,
  TrustDeviceEnrollReq,
  TrustDeviceEnrollRes,
  TrustDeviceVerifyChainReq,
  TrustDeviceVerifyChainRes,
  TrustDeviceListReq,
  TrustDeviceListRes,
  TrustProviderAddReq,
  TrustProviderRes,
  TrustProviderDisableReq,
  TrustProviderListReq,
  TrustProviderListRes,
  TrustPolicySetReq,
  TrustPolicySetRes,
  VaultLockReq,
  VaultLockRes,
  VaultLockStatusReq,
  VaultLockStatusRes,
  VaultUnlockReq,
  VaultUnlockRes,
  VaultEncryptionEnableReq,
  VaultEncryptionEnableRes,
  VaultEncryptionMigrateReq,
  VaultEncryptionMigrateRes,
  VaultRecoveryGenerateReq,
  VaultRecoveryGenerateRes,
  VaultRecoveryEscrowEnableReq,
  VaultRecoveryEscrowEnableRes,
  VaultRecoveryEscrowProviderAddReq,
  VaultRecoveryEscrowProviderAddRes,
  VaultRecoveryEscrowProviderListReq,
  VaultRecoveryEscrowProviderListRes,
  VaultRecoveryEscrowRestoreReq,
  VaultRecoveryEscrowRestoreRes,
  VaultRecoveryEscrowRotateReq,
  VaultRecoveryEscrowRotateAllReq,
  VaultRecoveryEscrowRotateAllRes,
  VaultRecoveryEscrowRotateRes,
  VaultRecoveryEscrowStatusReq,
  VaultRecoveryEscrowStatusRes,
  VaultRecoveryStatusReq,
  VaultRecoveryStatusRes,
  VaultRecoveryVerifyReq,
  VaultRecoveryVerifyRes,
  VaultEncryptionStatusReq,
  VaultEncryptionStatusRes,
  SyncPullReq,
  SyncPullRes,
  SyncMergePreviewReq,
  SyncMergePreviewRes,
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

export async function startTrustIdentity(
  api: DesktopRpcApi,
  req: TrustIdentityStartReq
): Promise<ViewState<TrustIdentityStartRes>> {
  return nextStateFromRpc(await api.trustIdentityStart(req));
}

export async function completeTrustIdentity(
  api: DesktopRpcApi,
  req: TrustIdentityCompleteReq
): Promise<ViewState<TrustIdentityCompleteRes>> {
  return nextStateFromRpc(await api.trustIdentityComplete(req));
}

export async function enrollTrustDevice(
  api: DesktopRpcApi,
  req: TrustDeviceEnrollReq
): Promise<ViewState<TrustDeviceEnrollRes>> {
  return nextStateFromRpc(await api.trustDeviceEnroll(req));
}

export async function verifyTrustDeviceChain(
  api: DesktopRpcApi,
  req: TrustDeviceVerifyChainReq
): Promise<ViewState<TrustDeviceVerifyChainRes>> {
  return nextStateFromRpc(await api.trustDeviceVerifyChain(req));
}

export async function listTrustDevices(
  api: DesktopRpcApi,
  req: TrustDeviceListReq
): Promise<ViewState<TrustDeviceListRes>> {
  return nextStateFromRpc(await api.trustDeviceList(req));
}

export async function addTrustProvider(
  api: DesktopRpcApi,
  req: TrustProviderAddReq
): Promise<ViewState<TrustProviderRes>> {
  return nextStateFromRpc(await api.trustProviderAdd(req));
}

export async function disableTrustProvider(
  api: DesktopRpcApi,
  req: TrustProviderDisableReq
): Promise<ViewState<TrustProviderRes>> {
  return nextStateFromRpc(await api.trustProviderDisable(req));
}

export async function listTrustProviders(
  api: DesktopRpcApi,
  req: TrustProviderListReq
): Promise<ViewState<TrustProviderListRes>> {
  return nextStateFromRpc(await api.trustProviderList(req));
}

export async function setTrustProviderPolicy(
  api: DesktopRpcApi,
  req: TrustPolicySetReq
): Promise<ViewState<TrustPolicySetRes>> {
  return nextStateFromRpc(await api.trustPolicySet(req));
}

export async function loadVaultEncryptionStatus(
  api: DesktopRpcApi,
  req: VaultEncryptionStatusReq
): Promise<ViewState<VaultEncryptionStatusRes>> {
  return nextStateFromRpc(await api.vaultEncryptionStatus(req));
}

export async function loadVaultLockStatus(
  api: DesktopRpcApi,
  req: VaultLockStatusReq
): Promise<ViewState<VaultLockStatusRes>> {
  return nextStateFromRpc(await api.vaultLockStatus(req));
}

export async function unlockVault(
  api: DesktopRpcApi,
  req: VaultUnlockReq
): Promise<ViewState<VaultUnlockRes>> {
  return nextStateFromRpc(await api.vaultUnlock(req));
}

export async function lockVault(
  api: DesktopRpcApi,
  req: VaultLockReq
): Promise<ViewState<VaultLockRes>> {
  return nextStateFromRpc(await api.vaultLock(req));
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

export async function loadVaultRecoveryStatus(
  api: DesktopRpcApi,
  req: VaultRecoveryStatusReq
): Promise<ViewState<VaultRecoveryStatusRes>> {
  return nextStateFromRpc(await api.vaultRecoveryStatus(req));
}

export async function generateVaultRecovery(
  api: DesktopRpcApi,
  req: VaultRecoveryGenerateReq
): Promise<ViewState<VaultRecoveryGenerateRes>> {
  return nextStateFromRpc(await api.vaultRecoveryGenerate(req));
}

export async function loadVaultRecoveryEscrowStatus(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowStatusReq
): Promise<ViewState<VaultRecoveryEscrowStatusRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowStatus(req));
}

export async function enableVaultRecoveryEscrow(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowEnableReq
): Promise<ViewState<VaultRecoveryEscrowEnableRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowEnable(req));
}

export async function rotateVaultRecoveryEscrow(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowRotateReq
): Promise<ViewState<VaultRecoveryEscrowRotateRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowRotate(req));
}

export async function restoreVaultRecoveryEscrow(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowRestoreReq
): Promise<ViewState<VaultRecoveryEscrowRestoreRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowRestore(req));
}

export async function addVaultRecoveryEscrowProvider(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowProviderAddReq
): Promise<ViewState<VaultRecoveryEscrowProviderAddRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowProviderAdd(req));
}

export async function listVaultRecoveryEscrowProviders(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowProviderListReq
): Promise<ViewState<VaultRecoveryEscrowProviderListRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowProviderList(req));
}

export async function rotateAllVaultRecoveryEscrow(
  api: DesktopRpcApi,
  req: VaultRecoveryEscrowRotateAllReq
): Promise<ViewState<VaultRecoveryEscrowRotateAllRes>> {
  return nextStateFromRpc(await api.vaultRecoveryEscrowRotateAll(req));
}

export async function verifyVaultRecovery(
  api: DesktopRpcApi,
  req: VaultRecoveryVerifyReq
): Promise<ViewState<VaultRecoveryVerifyRes>> {
  return nextStateFromRpc(await api.vaultRecoveryVerify(req));
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

export async function loadSyncMergePreview(
  api: DesktopRpcApi,
  req: SyncMergePreviewReq
): Promise<ViewState<SyncMergePreviewRes>> {
  return nextStateFromRpc(await api.syncMergePreview(req));
}
