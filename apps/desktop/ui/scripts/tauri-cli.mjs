import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const tauriBin = resolve("node_modules/.bin/tauri");
const args = process.argv.slice(2);
const tauriArgs = args;

const result = spawnSync(tauriBin, tauriArgs, {
  stdio: "inherit",
  cwd: resolve("../src-tauri")
});
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
