import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";
import { createI18n } from "./shared/i18n";

describe("App", () => {
  it("greets the user in the active language", async () => {
    // Arrange
    const i18n = await createI18n("es");

    // Act
    render(<App i18n={i18n} />);

    // Assert
    expect(
      screen.getByRole("heading", { name: "Bienvenido a Jade" }),
    ).toBeInTheDocument();
  });
});
