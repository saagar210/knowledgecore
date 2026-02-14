import type {
  DesktopRpcApi,
  EventsListReq,
  EventsListRes,
  JobsListReq,
  JobsListRes
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function listEvents(
  api: DesktopRpcApi,
  req: EventsListReq
): Promise<ViewState<EventsListRes>> {
  return nextStateFromRpc(await api.eventsList(req));
}

export async function listJobs(
  api: DesktopRpcApi,
  req: JobsListReq
): Promise<ViewState<JobsListRes>> {
  return nextStateFromRpc(await api.jobsList(req));
}
