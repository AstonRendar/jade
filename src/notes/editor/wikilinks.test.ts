import { CompletionContext } from "@codemirror/autocomplete";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { completeNote, wikilinkAt } from "./wikilinks";

describe("wikilinkAt", () => {
  it("finds the target of the link under the cursor", () => {
    // Arrange
    const line = "see [[Plan#Goals|the plan]] and [[Ideas]]";

    // Act
    const targets = [
      wikilinkAt(line, 8),
      wikilinkAt(line, 35),
      wikilinkAt(line, 1),
    ];

    // Assert
    expect(targets).toEqual(["Plan", "Ideas", null]);
  });
});

describe("completeNote", () => {
  it("offers note titles after an opening [[", () => {
    // Arrange
    const context = contextAt("see [[Pl");

    // Act
    const result = completeNote(context, ["projects/Plan.md", "Ideas.md"]);

    // Assert
    expect(result?.from).toBe(6);
    expect(
      result?.options.map((option) => [option.label, option.detail]),
    ).toEqual([
      ["Plan", "projects"],
      ["Ideas", undefined],
    ]);
  });

  it("offers nothing outside a wikilink", () => {
    // Arrange
    const context = contextAt("plain text");

    // Act
    const result = completeNote(context, ["Plan.md"]);

    // Assert
    expect(result).toBeNull();
  });

  it("closes the link when inserting a title", () => {
    // Arrange
    const view = new EditorView({
      state: EditorState.create({ doc: "see [[Pl" }),
    });
    const [option] =
      completeNote(contextAt("see [[Pl"), ["Plan.md"])?.options ?? [];

    // Act
    if (option && typeof option.apply === "function") {
      option.apply(view, option, 6, 8);
    }

    // Assert
    expect(view.state.doc.toString()).toBe("see [[Plan]]");
    view.destroy();
  });

  it("does not duplicate an already closed link", () => {
    // Arrange
    const view = new EditorView({
      state: EditorState.create({ doc: "see [[Pl]]" }),
    });
    const [option] =
      completeNote(contextAt("see [[Pl"), ["Plan.md"])?.options ?? [];

    // Act
    if (option && typeof option.apply === "function") {
      option.apply(view, option, 6, 8);
    }

    // Assert
    expect(view.state.doc.toString()).toBe("see [[Plan]]");
    view.destroy();
  });
});

function contextAt(text: string): CompletionContext {
  return new CompletionContext(
    EditorState.create({ doc: text }),
    text.length,
    false,
  );
}
