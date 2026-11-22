import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import type { ErrorView } from "../bindings/ErrorView";

export async function call<T>(command: string, args?: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toErrorView(error);
  }
}

export function isErrorView(value: unknown): value is ErrorView {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "params" in value
  );
}

function toErrorView(error: unknown): ErrorView {
  if (isErrorView(error)) {
    return error;
  }
  return { code: "unexpected", params: { detail: String(error) } };
}
