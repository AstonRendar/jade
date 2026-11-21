import { describe, expect, it } from "vitest";
import { resources, SUPPORTED_LOCALES } from "./resources";

type Tree = { [key: string]: string | Tree };

function entriesOf(tree: Tree, prefix = ""): [string, string][] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === "string"
      ? [[`${prefix}${key}`, value] as [string, string]]
      : entriesOf(value, `${prefix}${key}.`),
  );
}

function keysOf(tree: Tree): string[] {
  return entriesOf(tree)
    .map(([key]) => key)
    .sort();
}

describe("locales", () => {
  const referenceKeys = keysOf(resources.en.translation);

  it.each(SUPPORTED_LOCALES)("%s has exactly the keys of en", (locale) => {
    // Arrange
    const translation = resources[locale].translation;

    // Act
    const keys = keysOf(translation);

    // Assert
    expect(keys).toEqual(referenceKeys);
  });

  it.each(SUPPORTED_LOCALES)("%s has no empty translations", (locale) => {
    // Arrange
    const translation = resources[locale].translation;

    // Act
    const emptyKeys = entriesOf(translation)
      .filter(([, value]) => value.trim() === "")
      .map(([key]) => key);

    // Assert
    expect(emptyKeys).toEqual([]);
  });
});
