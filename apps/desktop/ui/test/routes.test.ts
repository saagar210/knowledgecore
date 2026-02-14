import { describe, expect, it } from "vitest";
import { appRoutes, routeDescriptors } from "../src/routes";

describe("route coverage", () => {
  it("includes all feature routes", () => {
    expect(appRoutes).toContain("vault");
    expect(appRoutes).toContain("ask");
    expect(appRoutes).toContain("verify");
    expect(appRoutes.length).toBeGreaterThanOrEqual(10);
  });

  it("has descriptors for each route", () => {
    expect(routeDescriptors.length).toBe(appRoutes.length);
    expect(routeDescriptors.find((d) => d.id === "ask")?.rpcMethod).toBe("askQuestion");
  });
});
