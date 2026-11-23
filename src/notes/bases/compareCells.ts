import type { CellView } from "../../shared/bindings/CellView";

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

export function compareCells(
  a: CellView | undefined,
  b: CellView | undefined,
): number {
  const left = sortable(a);
  const right = sortable(b);
  if (left === null || right === null) {
    return Number(left === null) - Number(right === null);
  }
  if (typeof left === "number" && typeof right === "number") {
    return left - right;
  }
  return collator.compare(String(left), String(right));
}

function sortable(cell: CellView | undefined): string | number | null {
  switch (cell?.type) {
    case undefined:
    case "null":
      return null;
    case "boolean":
      return Number(cell.value);
    case "number":
      return cell.value;
    case "date":
      return cell.milliseconds;
    case "text":
      return cell.value;
    case "link":
      return cell.label ?? cell.target;
    case "file":
      return cell.path;
    case "list":
      return cell.items.length === 0 ? null : sortable(cell.items[0]);
  }
}
