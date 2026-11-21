import { describe, expect, it } from "vitest";
import { resolveLocale } from "./resolveLocale";

describe("resolveLocale", () => {
  it("picks the first supported language in order of preference", () => {
    // Arrange
    const preferred = ["fr-FR", "ca-ES", "es-ES"];

    // Act
    const locale = resolveLocale(preferred);

    // Assert
    expect(locale).toBe("ca");
  });

  it("maps simplified Chinese variants to zh-CN", () => {
    // Arrange
    const variants = ["zh", "zh-CN", "zh-Hans", "zh-Hans-CN", "zh-SG"];

    // Act
    const locales = variants.map((variant) => resolveLocale([variant]));

    // Assert
    expect(locales).toEqual(["zh-CN", "zh-CN", "zh-CN", "zh-CN", "zh-CN"]);
  });

  it("does not treat traditional Chinese as simplified", () => {
    // Arrange
    const preferred = ["zh-TW", "zh-Hant", "zh-HK"];

    // Act
    const locale = resolveLocale(preferred);

    // Assert
    expect(locale).toBe("en");
  });

  it("is case-insensitive and accepts underscores", () => {
    // Arrange
    const preferred = ["ES_mx"];

    // Act
    const locale = resolveLocale(preferred);

    // Assert
    expect(locale).toBe("es");
  });

  it("falls back to English when nothing is supported", () => {
    // Arrange
    const preferred: string[] = [];

    // Act
    const locale = resolveLocale(preferred);

    // Assert
    expect(locale).toBe("en");
  });
});
