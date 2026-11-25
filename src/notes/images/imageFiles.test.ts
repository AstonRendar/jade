import { describe, expect, it } from "vitest";
import {
  imageEmbedsIn,
  imageSize,
  isImage,
  pastedImageName,
} from "./imageFiles";

describe("image files", () => {
  it("recognises image extensions", () => {
    // Arrange
    const paths = ["a.PNG", "b.jpeg", "c.webp", "d.svg", "e.md", "f.pdf"];

    // Act
    const images = paths.filter(isImage);

    // Assert
    expect(images).toEqual(["a.PNG", "b.jpeg", "c.webp", "d.svg"]);
  });

  it("finds image embeds and Markdown images with their size", () => {
    // Arrange
    const line =
      "![[Mapa.png|300]] ![[Nota]] ![[Foto.jpg|200x100]] ![alt](Recursos/Mi%20foto.webp) ![x](https://example.com/a.png) ![y](doc.pdf)";

    // Act
    const embeds = imageEmbedsIn(line);

    // Assert
    expect(embeds).toEqual([
      { target: "Mapa.png", width: 300, height: undefined },
      { target: "Foto.jpg", width: 200, height: 100 },
      { target: "Recursos/Mi foto.webp" },
      { target: "https://example.com/a.png" },
    ]);
  });

  it("reads sizes only from numbers", () => {
    // Arrange
    const aliases = [undefined, "caption", " 120 "];

    // Act
    const sizes = aliases.map(imageSize);

    // Assert
    expect(sizes).toEqual([{}, {}, { width: 120, height: undefined }]);
  });

  it("names pasted images like Obsidian", () => {
    // Arrange
    const now = new Date(2026, 10, 25, 9, 5, 7);

    // Act
    const names = [
      pastedImageName("image/png", now),
      pastedImageName("image/jpeg", now),
      pastedImageName("", now),
    ];

    // Assert
    expect(names).toEqual([
      "Pasted image 20261125090507.png",
      "Pasted image 20261125090507.jpg",
      "Pasted image 20261125090507.png",
    ]);
  });
});
