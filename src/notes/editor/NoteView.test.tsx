import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider } from "react-i18next";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createI18n } from "../../shared/i18n";
import { resetStores } from "../../shared/testing/resetStores";
import { useEditor } from "../store/editorStore";
import { useVault } from "../store/vaultStore";
import { NoteView } from "./NoteView";

describe("NoteView", () => {
  let followed: string[];

  beforeEach(() => {
    resetStores();
    followed = [];
    mockIPC(
      (command, args) => {
        const values = (args ?? {}) as Record<string, unknown>;
        if (command === "notes_read_note") {
          return values.path === "ideas.md"
            ? "# Ideas\n\nSee [[Plan]]."
            : "plan";
        }
        if (command === "notes_follow_link") {
          followed.push(values.target as string);
          return { path: "plan.md", created: false };
        }
        if (command === "notes_list_entries") {
          return [];
        }
        return null;
      },
      { shouldMockEvents: true },
    );
  });

  afterEach(async () => {
    cleanup();
    await new Promise((resolve) => setTimeout(resolve, 0));
    clearMocks();
  });

  it("shows the note title and rendered content in preview", async () => {
    // Arrange
    await renderNote("ideas.md");

    // Act
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));

    // Assert
    expect(
      screen.getByRole("heading", { level: 1, name: "ideas" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { level: 1, name: "Ideas" }),
    ).toBeInTheDocument();
  });

  it("follows a wikilink from the preview", async () => {
    // Arrange
    await renderNote("ideas.md");
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));

    // Act
    await userEvent.click(screen.getByRole("link", { name: "Plan" }));

    // Assert
    await waitFor(() => expect(useVault.getState().selected).toBe("plan.md"));
    expect(followed).toEqual(["Plan"]);
  });

  it("opens notes in the editor by default", async () => {
    // Arrange
    await renderNote("ideas.md");

    // Act
    const editing = screen.getByRole("button", { name: "Edit" });

    // Assert
    expect(editing).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("textbox")).toBeInTheDocument();
  });

  async function renderNote(path: string) {
    const i18n = await createI18n("en");
    useVault.setState({
      vault: { path: "/vaults/work", name: "work" },
      selected: path,
    });
    await useEditor.getState().load(path);
    render(
      <I18nextProvider i18n={i18n}>
        <NoteView />
      </I18nextProvider>,
    );
  }
});
