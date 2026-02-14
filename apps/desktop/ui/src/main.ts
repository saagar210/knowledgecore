import { createDesktopRpcApi } from "./api/rpc";
import { createFeatureControllers } from "./features";
import { appRoutes, routeDescriptors } from "./routes";
import { initializeFeatureStates } from "./state/featureState";

export function appName(): string {
  return "KnowledgeCore Desktop";
}

export { appRoutes } from "./routes";
export { routeDescriptors } from "./routes";
export { nextStateFromRpc } from "./state/appState";
export { initializeFeatureStates } from "./state/featureState";

export function createDesktopApp() {
  const api = createDesktopRpcApi();
  return {
    name: appName(),
    routes: appRoutes,
    routeDescriptors,
    featureStates: initializeFeatureStates(),
    controllers: createFeatureControllers(api)
  };
}
