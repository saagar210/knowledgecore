import type {
  DesktopRpcApi,
  JobsListReq,
  JobsListRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function loadSettingsDependencies(
  api: DesktopRpcApi,
  req: JobsListReq
): Promise<ViewState<JobsListRes>> {
  return nextStateFromRpc(await api.jobsList(req));
}
