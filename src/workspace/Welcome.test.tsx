import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { CapabilitiesView } from "../shared/bindings/CapabilitiesView";
import { createI18n } from "../shared/i18n";
import { resetStores } from "../shared/testing/resetStores";
import { Workspace } from "./Workspace";

type Call = { command: string; args: Record<string, unknown> };

const DESKTOP: CapabilitiesView = {
  canOpenFolders: true,
  managedVaultsFolder: null,
};
const MOBILE: CapabilitiesView = {
  canOpenFolders: false,
  managedVaultsFolder: "/data/jade/vaults",
};

describe("welcome screen on each platform", () => {
  let calls: Call[];
  let capabilities: CapabilitiesView;

  beforeEach(() => {
    resetStores();
    calls = [];
    capabilities = DESKTOP;
    mockIPC(
      (command, args) => {
        const values = (args ?? {}) as Record<string, unknown>;
        calls.push({ command, args: values });
        return respond(command, values);
      },
      { shouldMockEvents: true },
    );
  });

  afterEach(async () => {
    cleanup();
    await new Promise((resolve) => setTimeout(resolve, 0));
    clearMocks();
  });

  it("hides opening arbitrary folders on mobile", async () => {
    // Arrange
    capabilities = MOBILE;

    // Act
    await renderWorkspace();

    // Assert
    expect(
      await screen.findByRole("button", { name: "Create vault" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Open folder" }),
    ).not.toBeInTheDocument();
  });

  it("creates mobile vaults in the app storage", async () => {
    // Arrange
    capabilities = MOBILE;
    await renderWorkspace();

    // Act
    await userEvent.click(
      await screen.findByRole("button", { name: "Create vault" }),
    );
    await userEvent.type(screen.getByRole("textbox"), "Journal{Enter}");

    // Assert
    await waitFor(() =>
      expect(called("notes_create_vault")).toEqual([
        { parent: "/data/jade/vaults", name: "Journal" },
      ]),
    );
  });

  it("clones mobile vaults into the app storage without asking for a folder", async () => {
    // Arrange
    capabilities = MOBILE;
    await renderWorkspace();
    await userEvent.click(
      await screen.findByRole("button", { name: "Clone repository" }),
    );

    // Act
    await userEvent.type(
      screen.getByLabelText("Repository address"),
      "https://git.example.com/notes.git{Enter}",
    );

    // Assert
    await waitFor(() =>
      expect(called("sync_clone_repository")).toEqual([
        {
          url: "https://git.example.com/notes.git",
          parent: "/data/jade/vaults",
        },
      ]),
    );
    expect(called("plugin:dialog|open")).toEqual([]);
  });

  it("asks where to create a vault on desktop", async () => {
    // Arrange
    await renderWorkspace();

    // Act
    await userEvent.click(
      await screen.findByRole("button", { name: "Create vault" }),
    );
    await userEvent.type(screen.getByRole("textbox"), "Journal{Enter}");

    // Assert
    await waitFor(() =>
      expect(called("notes_create_vault")).toEqual([
        { parent: "/Users/example/Documents", name: "Journal" },
      ]),
    );
  });

  async function renderWorkspace() {
    const i18n = await createI18n("en");
    render(<Workspace i18n={i18n} />);
  }

  function called(command: string) {
    return calls
      .filter((call) => call.command === command)
      .map((call) => call.args);
  }

  function respond(command: string, args: Record<string, unknown>) {
    switch (command) {
      case "app_capabilities":
        return capabilities;
      case "preferences_get":
        return {
          locale: "en",
          theme: "system",
          autoSyncMinutes: 5,
          recentVaults: [],
        };
      case "plugin:dialog|open":
        return "/Users/example/Documents";
      case "notes_create_vault":
        return { path: `${args.parent}/${args.name}`, name: args.name };
      case "sync_clone_repository":
        return { path: `${args.parent}/notes`, name: "notes" };
      case "notes_list_entries":
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
