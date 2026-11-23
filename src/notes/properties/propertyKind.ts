import type { PropertySummaryView } from "../../shared/bindings/PropertySummaryView";
import type { PropertyTypeView } from "../../shared/bindings/PropertyTypeView";
import type { PropertyValueView } from "../../shared/bindings/PropertyValueView";

export type PropertyKind = PropertyTypeView | "object";

const LIST_KINDS: ReadonlySet<PropertyKind> = new Set([
  "list",
  "tags",
  "aliases",
]);

export function isListKind(kind: PropertyKind): boolean {
  return LIST_KINDS.has(kind);
}

export function kindOf(
  key: string,
  value: PropertyValueView,
  catalog: PropertySummaryView[],
): PropertyKind {
  if (value.type === "map") {
    return "object";
  }
  const known = catalog.find((summary) => summary.name === key);
  if (known) {
    return known.kind;
  }
  switch (value.type) {
    case "boolean":
      return "checkbox";
    case "number":
      return "number";
    case "list":
      return "list";
    default:
      return "text";
  }
}

export function emptyValue(kind: PropertyKind): PropertyValueView {
  if (isListKind(kind)) {
    return { type: "list", items: [] };
  }
  return kind === "checkbox"
    ? { type: "boolean", value: false }
    : { type: "null" };
}

export function textOf(value: PropertyValueView): string {
  switch (value.type) {
    case "null":
      return "";
    case "boolean":
    case "number":
    case "text":
      return String(value.value);
    case "list":
      return value.items.map(textOf).join(", ");
    case "map":
      return value.entries
        .map((entry) => `${entry.key}: ${textOf(entry.value)}`)
        .join(", ");
  }
}

export function itemsOf(value: PropertyValueView): string[] {
  if (value.type === "list") {
    return value.items.map(textOf).filter((item) => item !== "");
  }
  const text = textOf(value);
  return text === "" ? [] : [text];
}

export function textValue(text: string): PropertyValueView {
  return text === "" ? { type: "null" } : { type: "text", value: text };
}

export function wikilinkTarget(text: string): string | null {
  const match = /^\[\[([^\]|#]+)(?:[#|][^\]]*)?\]\]$/.exec(text.trim());
  return match?.[1]?.trim() ?? null;
}
