import { describe, expect, it } from "vitest";
import type { EntryView } from "../../shared/bindings/EntryView";
import { buildTree } from "./buildTree";

describe("buildTree", () => {
  it("nests entries under their folders", () => {
    // Arrange
    const entries: EntryView[] = [
      { path: "projects", kind: "folder" },
      { path: "projects/plan.md", kind: "note" },
      { path: "ideas.md", kind: "note" },
    ];

    // Act
    const tree = buildTree(entries);

    // Assert
    expect(tree.map((node) => node.path)).toEqual(["projects", "ideas.md"]);
    expect(tree[0]?.children.map((node) => node.path)).toEqual([
      "projects/plan.md",
    ]);
  });

  it("lists folders first, then names in natural order", () => {
    // Arrange
    const entries: EntryView[] = [
      { path: "note 10.md", kind: "note" },
      { path: "note 2.md", kind: "note" },
      { path: "zeta", kind: "folder" },
      { path: "alpha.png", kind: "file" },
    ];

    // Act
    const names = buildTree(entries).map((node) => node.name);

    // Assert
    expect(names).toEqual(["zeta", "alpha.png", "note 2", "note 10"]);
  });

  it("shows notes without their extension", () => {
    // Arrange
    const entries: EntryView[] = [{ path: "Meeting.md", kind: "note" }];

    // Act
    const [node] = buildTree(entries);

    // Assert
    expect(node?.name).toBe("Meeting");
  });
});
