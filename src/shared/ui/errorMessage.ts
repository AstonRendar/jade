import type { TFunction } from "i18next";
import type { ErrorView } from "../bindings/ErrorView";

export function errorMessage(t: TFunction, error: ErrorView): string {
  return t(`errors.${error.code}`, error.params);
}
