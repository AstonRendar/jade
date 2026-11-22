import { describe, expect, it } from "vitest";
import { createI18n } from "../i18n";
import { errorMessage } from "./errorMessage";

describe("errorMessage", () => {
  it("translates the error code with its parameters", async () => {
    // Arrange
    const i18n = await createI18n("es");

    // Act
    const message = errorMessage(i18n.t, {
      code: "already_exists",
      params: { path: "Ideas.md" },
    });

    // Assert
    expect(message).toBe("Ya existe «Ideas.md».");
  });
});
