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

  it("renders callouts with their kind, icon and title", () => {
    // Arrange
    const source = "> [!warning] Mind the gap\n> Body **text**";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain('<div class="callout" data-callout="warning">');
    expect(html).toContain("Mind the gap</span></div>");
    expect(html).toContain("<p>Body <strong>text</strong></p>");
    expect(html).not.toContain("[!warning]");
  });

  it("renders foldable callouts, aliases and callouts without a body", () => {
    // Arrange
    const sources = [
      "> [!faq]- Why?\n> Because.",
      "> [!tip]+\n> Open by default",
      "> [!unknown] Only a title",
    ];

    // Act
    const [folded, open, bare] = sources.map((source) =>
      renderMarkdown(source),
    );

    // Assert
    expect(folded).toContain(
      '<details class="callout" data-callout="question">',
    );
    expect(folded).not.toContain(" open>");
    expect(open).toContain('data-callout="tip" open>');
    expect(open).toContain("Tip</span>");
    expect(bare).toContain('data-callout="note"');
    expect(bare).toContain("Only a title");
    expect(bare).not.toContain("<p>");
  });

  it("keeps plain quotes as quotes", () => {
    // Arrange
    const source = "> Just a quote";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toBe("<blockquote>\n<p>Just a quote</p>\n</blockquote>\n");
  });

  it("renders task lists with checkboxes", () => {
    // Arrange
    const source = "- [ ] Write\n- [x] Publish\n- Plain";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain(
      '<li class="task-item"><input class="task" type="checkbox" disabled> Write</li>',
    );
    expect(html).toContain(
      '<li class="task-item done"><input class="task" type="checkbox" disabled checked> Publish</li>',
    );
    expect(html).toContain("<li>Plain</li>");
  });

  it("renders highlights and hides comments", () => {
    // Arrange
    const source =
      "A ==marked **word**== and %%hidden%% text\n\n%%\nblock\ncomment\n%%\n\nAfter == not == marked";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain("<mark>marked <strong>word</strong></mark>");
    expect(html).toContain(
      "A <mark>marked <strong>word</strong></mark> and text",
    );
    expect(html).not.toContain("hidden");
    expect(html).not.toContain("block");
    expect(html).toContain("After == not == marked");
  });

  it("renders tags but not headings, numbers or anchors", () => {
    // Arrange
    const source =
      "#idea and #saga/norte/ in text, issue #12, [[Note#Part]], `#code`";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain('<span class="tag">#idea</span>');
    expect(html).toContain('<span class="tag">#saga/norte</span>');
    expect(html).toContain("issue #12");
    expect(html).toContain("<code>#code</code>");
    expect(html.match(/class="tag"/g)).toHaveLength(2);
  });

  it("wraps tables and labels code blocks with their language", () => {
    // Arrange
    const source =
      "| a | b |\n|---|---|\n| 1 | 2 |\n\n```rust\nfn main() {}\n```\n\n```\nplain\n```";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain('<div class="table"><table>');
    expect(html).toContain(
      '<div class="code-block"><span class="code-language">rust</span><pre>',
    );
    expect(html).toContain("<pre><code>plain");
  });

  it("marks embeds so they render as cards", () => {
    // Arrange
    const source = "![[Capítulo 1]] and [[Plan]]";

    // Act
    const html = renderMarkdown(source);

    // Assert
    expect(html).toContain(
      '<a class="wikilink embed" href="#" data-target="Capítulo 1">',
    );
    expect(html).toContain('<a class="wikilink" href="#" data-target="Plan">');
    expect(html).not.toContain("!<a");
  });

  it("shows image embeds and Markdown images from their resolved files", () => {
    // Arrange
    const source =
      "![[Portada.webp|300]] ![[Mapa.png|200x100]] ![alt text](Recursos/Mi%20mapa.png) ![web](https://example.com/a.png)";
    const images = {
      "Portada.webp": "asset://Recursos/Portada.webp",
      "Mapa.png": "asset://Mapa.png",
      "Recursos/Mi mapa.png": "asset://Recursos/Mi%20mapa.png",
    };

    // Act
    const html = renderMarkdown(source, images);

    // Assert
    expect(html).toContain(
      '<img class="embed-image" src="asset://Recursos/Portada.webp" alt="" data-image="Portada.webp" width="300" loading="lazy">',
    );
    expect(html).toContain('data-image="Mapa.png" width="200" height="100"');
    expect(html).toContain('alt="alt text" data-image="Recursos/Mi mapa.png"');
    expect(html).toContain('src="https://example.com/a.png" alt="web"');
  });

  it("shows the name of images that cannot be found", () => {
    // Arrange
    const source = "![[Missing.png]] ![x](nope.jpg)";

    // Act
    const html = renderMarkdown(source, {});

    // Assert
    expect(html).toContain(
      '<span class="embed-image missing" data-image="Missing.png">Missing.png</span>',
    );
    expect(html).toContain('data-image="nope.jpg">nope.jpg</span>');
  });
});
