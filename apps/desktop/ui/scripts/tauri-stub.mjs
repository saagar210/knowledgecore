const args = process.argv.slice(2);
if (args[0] !== "build") {
  console.error("Only 'build' is supported in scaffold mode.");
  process.exit(1);
}
console.log("tauri build scaffold succeeded");
