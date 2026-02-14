import { describe, expect, it } from "vitest";
import { appName, createDesktopApp } from "../src/main";

describe("ui scaffold", () => {
  it("returns app name", () => {
    expect(appName()).toBe("KnowledgeCore Desktop");
  });

  it("creates full desktop app wiring for all feature routes", () => {
    const app = createDesktopApp();
    expect(app.routes.length).toBe(11);
    expect(Object.keys(app.controllers).sort()).toEqual([
      "ask",
      "document",
      "events",
      "export",
      "ingest",
      "lineage",
      "related",
      "search",
      "settings",
      "vault",
      "verify"
    ]);
  });
});
