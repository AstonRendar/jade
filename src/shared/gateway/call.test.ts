import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";
import { call } from "./call";

describe("call", () => {
  afterEach(clearMocks);

  it("returns what the command returns", async () => {
    // Arrange
    mockIPC(() => ["a.md"]);

    // Act
    const result = await call<string[]>("notes_list_entries");

    // Assert
    expect(result).toEqual(["a.md"]);
  });

  it("rejects with the error view sent by the core", async () => {
    // Arrange
    mockIPC(() => {
      throw { code: "vault_not_open", params: {} };
    });

    // Act
    const result = call("notes_list_entries");

    // Assert
    await expect(result).rejects.toEqual({
      code: "vault_not_open",
      params: {},
    });
  });

  it("wraps unknown failures as unexpected errors", async () => {
    // Arrange
    mockIPC(() => {
      throw "boom";
    });

    // Act
    const result = call("notes_list_entries");

    // Assert
    await expect(result).rejects.toEqual({
      code: "unexpected",
      params: { detail: "boom" },
    });
  });
});
