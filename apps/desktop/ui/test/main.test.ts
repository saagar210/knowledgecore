import { describe, expect, it } from "vitest";
import { appName } from "../src/main";

describe("ui scaffold", () => {
  it("returns app name", () => {
    expect(appName()).toBe("KnowledgeCore Desktop");
  });
});
