import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { EntryView } from "../shared/bindings/EntryView";
import type { PreferencesView } from "../shared/bindings/PreferencesView";
import { createI18n } from "../shared/i18n";
import { resetStores } from "../shared/testing/resetStores";
import { Workspace } from "./Workspace";

type Calls = { command: string; args: Record<string, unknown> }[];

describe("Workspace", () => {
  let calls: Calls;
  let entries: EntryView[];
  let preferences: PreferencesView;
  let missingVaults: string[];

  beforeEach(() => {
    resetStores();
    calls = [];
    entries = [
      { path: "projects", kind: "folder" },
      { path: "projects/plan.md", kind: "note" },
      { path: "ideas.md", kind: "note" },
    ];
    preferences = {
      locale: "en",
      theme: "system",
      autoSyncMinutes: 5,
      recentVaults: ["/vaults/work", "/vaults/gone"],
    };
    missingVaults = ["/vaults/gone"];
    mockIPC(
      (command, args) => {
        calls.push({ command, args: (args ?? {}) as Record<string, unknown> });
        return respond(command, (args ?? {}) as Record<string, unknown>);
      },
      { shouldMockEvents: true },
    );
  });

  afterEach(async () => {
    cleanup();
    await new Promise((resolve) => setTimeout(resolve, 0));
    clearMocks();
  });

  it("offers the recently opened vaults", async () => {
    // Arrange
    await renderWorkspace();

    // Act
    const recent = await screen.findByRole("button", { name: /work/ });

    // Assert
    expect(recent).toBeInTheDocument();
  });

  it("opens a recent vault and shows its notes", async () => {
    // Arrange
    await renderWorkspace();

    // Act
    await userEvent.click(await screen.findByRole("button", { name: /work/ }));

    // Assert
    expect(
      await screen.findByRole("treeitem", { name: "ideas" }),
    ).toBeInTheDocument();
    expect(called("notes_open_vault")).toEqual([{ path: "/vaults/work" }]);
  });

  it("forgets a recent vault that no longer exists", async () => {
    // Arrange
    await renderWorkspace();

    // Act
    await userEvent.click(await screen.findByRole("button", { name: /gone/ }));

    // Assert
    expect(
      await screen.findByText("The vault folder no longer exists."),
    ).toBeInTheDocument();
    expect(called("preferences_forget_vault")).toEqual([
      { path: "/vaults/gone" },
    ]);
  });

  it("creates a note inside the folder chosen in the tree", async () => {
    // Arrange
    await openWorkVault();
    await userEvent.pointer({
      keys: "[MouseRight]",
      target: screen.getByRole("treeitem", { name: "projects" }),
    });

    // Act
    await userEvent.click(screen.getByRole("menuitem", { name: "New note" }));
    await userEvent.type(screen.getByRole("textbox"), "Roadmap{Enter}");

    // Assert
    await waitFor(() =>
      expect(called("notes_create_note")).toEqual([
        { folder: "projects", title: "Roadmap" },
      ]),
    );
  });

  it("asks before deleting and then deletes", async () => {
    // Arrange
    await openWorkVault();
    await userEvent.pointer({
      keys: "[MouseRight]",
      target: screen.getByRole("treeitem", { name: "ideas" }),
    });

    // Act
    await userEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
    await userEvent.click(screen.getByRole("button", { name: "Delete" }));

    // Assert
    await waitFor(() =>
      expect(called("notes_delete_entry")).toEqual([{ path: "ideas.md" }]),
    );
  });

  it("renames keeping the note extension", async () => {
    // Arrange
    await openWorkVault();
    await userEvent.pointer({
      keys: "[MouseRight]",
      target: screen.getByRole("treeitem", { name: "plan" }),
    });

    // Act
    await userEvent.click(screen.getByRole("menuitem", { name: "Rename" }));
    const input = screen.getByRole("textbox");
    await userEvent.clear(input);
    await userEvent.type(input, "Roadmap{Enter}");

    // Assert
    await waitFor(() =>
      expect(called("notes_rename_entry")).toEqual([
        { from: "projects/plan.md", to: "projects/Roadmap.md" },
      ]),
    );
  });

  it("selects a note when it is clicked", async () => {
    // Arrange
    await openWorkVault();

    // Act
    await userEvent.click(screen.getByRole("treeitem", { name: "ideas" }));

    // Assert
    expect(screen.getByRole("treeitem", { name: "ideas" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  async function renderWorkspace() {
    const i18n = await createI18n("en");
    render(<Workspace i18n={i18n} />);
  }

  async function openWorkVault() {
    await renderWorkspace();
    await userEvent.click(await screen.findByRole("button", { name: /work/ }));
    await screen.findByRole("treeitem", { name: "ideas" });
    await userEvent.click(screen.getByRole("treeitem", { name: "projects" }));
  }

  function called(command: string) {
    return calls
      .filter((call) => call.command === command)
      .map((call) => call.args);
  }

  function respond(command: string, args: Record<string, unknown>) {
    switch (command) {
      case "preferences_get":
        return preferences;
      case "preferences_forget_vault":
        preferences = { ...preferences, recentVaults: ["/vaults/work"] };
        return preferences;
      case "notes_open_vault":
        if (missingVaults.includes(args.path as string)) {
          throw { code: "vault_not_found", params: {} };
        }
        return { path: args.path, name: "work" };
      case "notes_list_entries":
        return entries;
      case "notes_create_note":
        return `${args.folder}/${args.title}.md`;
      case "notes_read_properties":
        return { properties: [], bodyStart: 0, valid: true };
      case "notes_list_properties":
        return [];
      case "notes_read_note":
        return "";
      case "notes_find_backlinks":
      case "sync_list_note_history":
        return [];
      case "sync_get_status":
        return {
          isRepository: false,
          branch: null,
          remote: null,
          pendingChanges: 0,
          ahead: 0,
          behind: 0,
        };
      default:
        return null;
    }
  }
});
