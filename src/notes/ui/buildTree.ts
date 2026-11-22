import type { EntryKindView } from "../../shared/bindings/EntryKindView";
import type { EntryView } from "../../shared/bindings/EntryView";

export type TreeNode = {
  path: string;
  name: string;
  kind: EntryKindView;
  children: TreeNode[];
};

const NOTE_EXTENSION = /\.md$/i;
const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

export function buildTree(entries: EntryView[]): TreeNode[] {
  const nodes = new Map<string, TreeNode>();
  const roots: TreeNode[] = [];
  const byDepth = [...entries].sort(
    (a, b) => a.path.split("/").length - b.path.split("/").length,
  );
  for (const entry of byDepth) {
    const node = toNode(entry);
    nodes.set(entry.path, node);
    const parent = nodes.get(parentOf(entry.path));
    (parent ? parent.children : roots).push(node);
  }
  return sortTree(roots);
}

export function parentOf(path: string): string {
  const index = path.lastIndexOf("/");
  return index === -1 ? "" : path.slice(0, index);
}

function toNode(entry: EntryView): TreeNode {
  const fileName = entry.path.slice(entry.path.lastIndexOf("/") + 1);
  return {
    path: entry.path,
    kind: entry.kind,
    name:
      entry.kind === "note" ? fileName.replace(NOTE_EXTENSION, "") : fileName,
    children: [],
  };
}

function sortTree(nodes: TreeNode[]): TreeNode[] {
  return nodes
    .map((node) => ({ ...node, children: sortTree(node.children) }))
    .sort(
      (a, b) =>
        Number(b.kind === "folder") - Number(a.kind === "folder") ||
        collator.compare(a.name, b.name),
    );
}
