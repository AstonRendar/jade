import { describe, expect, it } from "vitest";
import { renderMarkdown } from "./renderMarkdown";

describe("renderMarkdown", () => {
  it("renders wikilinks as links to their target", () => {
    // Arrange
    const source = "See [[Plan#Goals|the plan]] and [[Ideas]].";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain(
      '<a class="wikilink" href="#" data-target="Plan">the plan</a>',
    );
    expect(html).toContain(
      '<a class="wikilink" href="#" data-target="Ideas">Ideas</a>',
    );
  });

  it("leaves wikilinks inside code untouched", () => {
    // Arrange
    const source = "`[[Plan]]`";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toBe("<p><code>[[Plan]]</code></p>\n");
  });

  it("escapes raw HTML", () => {
    // Arrange
    const source = "<script>alert(1)</script>";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).not.toContain("<script>");
  });

  it("renders ordinary Markdown", () => {
    // Arrange
    const source = "# Title\n\n- **bold** item";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain("<h1>Title</h1>");
    expect(html).toContain("<strong>bold</strong>");
  });
});
