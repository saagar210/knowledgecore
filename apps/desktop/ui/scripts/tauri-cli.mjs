import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const uiRoot = resolve(scriptDir, "..");
const tauriBin = resolve(uiRoot, "node_modules/.bin/tauri");
const args = process.argv.slice(2);
const tauriArgs = args;

if (tauriArgs[0] === "build") {
  const buildResult = spawnSync("pnpm", ["run", "build"], {
    stdio: "inherit",
    cwd: uiRoot
  });
  if (buildResult.error) {
    console.error(buildResult.error.message);
    process.exit(1);
  }
  if ((buildResult.status ?? 1) !== 0) {
    process.exit(buildResult.status ?? 1);
  }
}

const result = spawnSync(tauriBin, tauriArgs, {
  stdio: "inherit",
  cwd: resolve(uiRoot, "../src-tauri")
});
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
