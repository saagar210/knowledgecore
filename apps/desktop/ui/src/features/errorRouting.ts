import type { AppError } from "../api/rpc";

export type UiRoute =
  | "vault-setup"
  | "dependency-setup"
  | "fatal-error";

export function routeForError(error: AppError): UiRoute {
  if (error.code.startsWith("KC_VAULT_")) {
    return "vault-setup";
  }
  if (error.code === "KC_PDFIUM_UNAVAILABLE" || error.code === "KC_TESSERACT_UNAVAILABLE") {
    return "dependency-setup";
  }
  return "fatal-error";
}
