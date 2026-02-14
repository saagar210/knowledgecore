export type AppError = {
  schema_version: number;
  code: string;
  category: string;
  message: string;
  retryable: boolean;
  details: unknown;
};

export type RpcOk<T> = { ok: true; data: T };
export type RpcErr = { ok: false; error: AppError };
export type RpcResp<T> = RpcOk<T> | RpcErr;

export async function rpc<TReq, TRes>(cmd: string, req: TReq): Promise<RpcResp<TRes>> {
  void cmd;
  void req;
  return {
    ok: false,
    error: {
      schema_version: 1,
      code: "KC_RPC_NOT_WIRED",
      category: "rpc",
      message: "RPC invoke layer is not wired in this scaffold",
      retryable: true,
      details: {}
    }
  };
}
