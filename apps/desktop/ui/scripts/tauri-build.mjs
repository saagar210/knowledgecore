import { spawnSync } from "node:child_process";

const result = spawnSync("cargo", ["build", "--manifest-path", "../src-tauri/Cargo.toml"], {
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
