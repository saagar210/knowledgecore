import { describe, expect, it } from "vitest";
import { appRoutes } from "../src/routes";

describe("route coverage", () => {
  it("includes all feature routes", () => {
    expect(appRoutes).toContain("vault");
    expect(appRoutes).toContain("ask");
    expect(appRoutes).toContain("verify");
    expect(appRoutes.length).toBeGreaterThanOrEqual(10);
  });
});
