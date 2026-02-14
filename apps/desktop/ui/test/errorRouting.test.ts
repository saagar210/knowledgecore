import { describe, expect, it } from "vitest";
import type { AppError } from "../src/api/rpc";
import { routeForError } from "../src/features/errorRouting";

function err(code: string, retryable: boolean, message: string): AppError {
  return {
    schema_version: 1,
    code,
    category: "test",
    message,
    retryable,
    details: {}
  };
}

describe("error routing", () => {
  it("routes on code not message", () => {
    const a = err("KC_VAULT_JSON_MISSING", false, "any message");
    const b = err("KC_VAULT_JSON_MISSING", false, "totally different text");
    expect(routeForError(a)).toBe("vault-setup");
    expect(routeForError(b)).toBe("vault-setup");
  });

  it("routes by code only and ignores retryable flag", () => {
    expect(routeForError(err("KC_X", true, "m"))).toBe("fatal-error");
    expect(routeForError(err("KC_X", false, "m"))).toBe("fatal-error");
  });
});
