import { describe, expect, it } from "vitest";
import { matchingFiles } from "./SearchPanel";

const PATHS = [
  "Personajes/Iby.md",
  "Tramas/Vigilia/Sesión 7.md",
  "Lore/Las Torres.md",
  "Índice.base",
];

describe("matchingFiles", () => {
  it("matches names, best first, with their folder", () => {
    // Arrange
    const query = "torr";

    // Act
    const files = matchingFiles(PATHS, query);

    // Assert
    expect(files).toEqual([
      {
        path: "Lore/Las Torres.md",
        name: "Las Torres",
        folder: "Lore",
        positions: [4, 5, 6, 7],
      },
    ]);
  });

  it("matches folders only when the query names one", () => {
    // Arrange
    const queries = ["iby", "per/iby", "vigilia"];

    // Act
    const [byName, byFolder, folderWithoutSlash] = queries.map((query) =>
      matchingFiles(PATHS, query).map((file) => file.path),
    );

    // Assert
    expect(byName).toEqual(["Personajes/Iby.md"]);
    expect(byFolder).toEqual(["Personajes/Iby.md"]);
    expect(folderWithoutSlash).toEqual([]);
  });

  it("keeps the extension of files that are not notes", () => {
    // Arrange
    const query = "indice";

    // Act
    const files = matchingFiles(PATHS, query);

    // Assert
    expect(files.map((file) => [file.name, file.folder])).toEqual([
      ["Índice.base", ""],
    ]);
  });
});
