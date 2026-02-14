import type { DesktopRpcApi } from "../api/rpc";
import * as ask from "./ask";
import * as document from "./document";
import * as events from "./events";
import * as exportVerify from "./exportVerify";
import * as ingest from "./ingest";
import * as lineage from "./lineage";
import * as related from "./related";
import * as search from "./search";
import * as settings from "./settings";
import * as vault from "./vault";

export type FeatureControllers = {
  vault: typeof vault;
  ingest: typeof ingest;
  search: typeof search;
  document: typeof document;
  related: typeof related;
  ask: typeof ask;
  export: Pick<typeof exportVerify, "exportBundle">;
  verify: Pick<typeof exportVerify, "verifyBundle">;
  events: typeof events;
  settings: typeof settings;
  lineage: typeof lineage;
};

export function createFeatureControllers(_api: DesktopRpcApi): FeatureControllers {
  // Controllers intentionally stay thin and delegate to RPC-only methods.
  return {
    vault,
    ingest,
    search,
    document,
    related,
    ask,
    export: { exportBundle: exportVerify.exportBundle },
    verify: { verifyBundle: exportVerify.verifyBundle },
    events,
    settings,
    lineage
  };
}
