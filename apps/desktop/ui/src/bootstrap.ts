import { createDesktopApp } from "./main";

type TauriRuntime = {
  core?: { invoke?: unknown };
  tauri?: { invoke?: unknown };
};

function rpcWired(): boolean {
  const runtime = (globalThis as { __TAURI__?: TauriRuntime }).__TAURI__;
  return Boolean(runtime?.core?.invoke ?? runtime?.tauri?.invoke);
}

function render(): void {
  const mount = document.getElementById("app");
  if (!mount) return;

  const app = createDesktopApp();
  const routeRows = app.routeDescriptors
    .map((route) => {
      const state = app.featureStates[route.id];
      const rpc = route.rpcMethod ?? "n/a";
      return `<tr><td>${route.title}</td><td><code>${rpc}</code></td><td>${state?.status ?? "idle"}</td></tr>`;
    })
    .join("");

  const rpcState = rpcWired() ? "connected" : "not wired";

  mount.innerHTML = `
    <section style="font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 24px auto; max-width: 980px; color: #14223a;">
      <h1 style="margin: 0 0 8px; font-size: 28px;">${app.name}</h1>
      <p style="margin: 0 0 16px; color: #3d4e6a;">Desktop shell initialized. RPC bridge: <strong>${rpcState}</strong>.</p>
      <table style="width: 100%; border-collapse: collapse; background: #fff;">
        <thead>
          <tr style="text-align: left; border-bottom: 1px solid #dbe3ef;">
            <th style="padding: 10px;">Feature</th>
            <th style="padding: 10px;">RPC Method</th>
            <th style="padding: 10px;">State</th>
          </tr>
        </thead>
        <tbody>${routeRows}</tbody>
      </table>
    </section>
  `;
}

render();
