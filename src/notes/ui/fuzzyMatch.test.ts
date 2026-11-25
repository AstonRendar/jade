import { describe, expect, it } from "vitest";
import { fuzzyMatch } from "./fuzzyMatch";

describe("fuzzyMatch", () => {
  it("matches letters in order, ignoring case, accents and spaces", () => {
    // Arrange
    const text = "Tía Elena";

    // Act
    const match = fuzzyMatch("tia el", text);

    // Assert
    expect(match?.positions).toEqual([0, 1, 2, 4, 5]);
  });

  it("rejects text missing a letter or with letters out of order", () => {
    // Arrange
    const text = "Glosario";

    // Act
    const missing = fuzzyMatch("glz", text);
    const reversed = fuzzyMatch("og", text);

    // Assert
    expect(missing).toBeNull();
    expect(reversed).toBeNull();
  });

  it("rejects letters scattered across the middle of words", () => {
    // Arrange
    const scattered = "Teóforo apeador de lindes";

    // Act
    const noise = fuzzyMatch("torre", scattered);
    const oneJump = fuzzyMatch("pln", "plan");
    const acronym = fuzzyMatch("lt", "Las Torres");

    // Assert
    expect(noise).toBeNull();
    expect(oneJump?.positions).toEqual([0, 1, 3]);
    expect(acronym?.positions).toEqual([0, 4]);
  });

  it("prefers word starts, prefixes and consecutive letters", () => {
    // Arrange
    const candidates = ["Robert Denmark", "Tramas de Rd", "Rd"];

    // Act
    const scores = candidates.map(
      (text) => fuzzyMatch("rd", text)?.score ?? -Infinity,
    );

    // Assert
    expect(scores[2]).toBeGreaterThan(scores[0] ?? 0);
    expect(scores[0]).toBeGreaterThan(scores[1] ?? 0);
  });

  it("jumps to word starts when the same letter appears earlier inside a word", () => {
    // Arrange
    const text = "Personajes/Iby";

    // Act
    const match = fuzzyMatch("pi", text);

    // Assert
    expect(match?.positions).toEqual([0, 11]);
  });

  it("matches everything with an empty query", () => {
    // Arrange
    const query = "  ";

    // Act
    const match = fuzzyMatch(query, "Inicio");

    // Assert
    expect(match).toEqual({ score: 0, positions: [] });
  });
});
