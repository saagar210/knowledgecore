import type {
  AskQuestionReq,
  AskQuestionRes,
  DesktopRpcApi
} from "../api/rpc";
import { nextStateFromRpc, type ViewState } from "../state/appState";

export async function askQuestion(
  api: DesktopRpcApi,
  req: AskQuestionReq
): Promise<ViewState<AskQuestionRes>> {
  return nextStateFromRpc(await api.askQuestion(req));
}
