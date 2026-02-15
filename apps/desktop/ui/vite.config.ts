import { defineConfig } from "vite";

export default defineConfig({
  build: {
    outDir: "tauri-dist",
    emptyOutDir: true,
    sourcemap: false
  }
});
