export function appName(): string {
  return "KnowledgeCore Desktop";
}

export { appRoutes } from "./routes";
export { routeDescriptors } from "./routes";
export { nextStateFromRpc } from "./state/appState";
export { initializeFeatureStates } from "./state/featureState";
