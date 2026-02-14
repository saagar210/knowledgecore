export const appRoutes = [
  "vault",
  "ingest",
  "search",
  "document",
  "related",
  "ask",
  "export",
  "verify",
  "events",
  "settings"
] as const;

export type AppRoute = (typeof appRoutes)[number];

export type RouteDescriptor = {
  id: AppRoute;
  title: string;
  rpcMethod?: string;
};

export const routeDescriptors: RouteDescriptor[] = [
  { id: "vault", title: "Vault Management", rpcMethod: "vaultOpen" },
  { id: "ingest", title: "Ingest", rpcMethod: "ingestScanFolder" },
  { id: "search", title: "Search", rpcMethod: "searchQuery" },
  { id: "document", title: "Document Viewer", rpcMethod: "locatorResolve" },
  { id: "related", title: "Related Items", rpcMethod: "searchQuery" },
  { id: "ask", title: "Ask", rpcMethod: "askQuestion" },
  { id: "export", title: "Export", rpcMethod: "exportBundle" },
  { id: "verify", title: "Verifier", rpcMethod: "verifyBundle" },
  { id: "events", title: "Events", rpcMethod: "eventsList" },
  { id: "settings", title: "Settings", rpcMethod: "jobsList" }
];
