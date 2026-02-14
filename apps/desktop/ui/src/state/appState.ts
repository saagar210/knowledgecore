import type { RpcResp } from "../api/rpc";
import { routeForError, type UiRoute } from "../features/errorRouting";

export type ViewState<T> =
  | { kind: "idle" }
  | { kind: "data"; value: T }
  | { kind: "error"; route: UiRoute; code: string };

export function nextStateFromRpc<T>(resp: RpcResp<T>): ViewState<T> {
  if (resp.ok) {
    return { kind: "data", value: resp.data };
  }
  return {
    kind: "error",
    route: routeForError(resp.error),
    code: resp.error.code
  };
}
