import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createI18n } from "../shared/i18n";
import { resetStores } from "../shared/testing/resetStores";
import { Workspace } from "./Workspace";

describe("navigation drawer on narrow screens", () => {
  beforeEach(() => {
    resetStores();
    mockIPC((command) => respond(command), { shouldMockEvents: true });
  });

  afterEach(async () => {
    cleanup();
    await new Promise((resolve) => setTimeout(resolve, 0));
    clearMocks();
  });

  it("opens the navigation from the menu button", async () => {
    // Arrange
    await openVault();

    // Act
    await userEvent.click(screen.getByRole("button", { name: "Show notes" }));

    // Assert
    expect(shell()).toHaveAttribute("data-navigation", "open");
  });

  it("closes the navigation when a note is chosen", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Show notes" }));

    // Act
    await userEvent.click(screen.getByRole("treeitem", { name: "ideas" }));

    // Assert
    await waitFor(() =>
      expect(shell()).toHaveAttribute("data-navigation", "closed"),
    );
  });

  it("closes the navigation when tapping outside it", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Show notes" }));

    // Act
    await userEvent.click(screen.getByTestId("navigation-backdrop"));

    // Assert
    expect(shell()).toHaveAttribute("data-navigation", "closed");
  });

  async function openVault() {
    const i18n = await createI18n("en");
    render(<Workspace i18n={i18n} />);
    await userEvent.click(await screen.findByRole("button", { name: /work/ }));
    await screen.findByRole("treeitem", { name: "ideas" });
  }

  function shell() {
    return screen.getByTestId("app-shell");
  }

  function respond(command: string) {
    switch (command) {
      case "preferences_get":
        return {
          locale: "en",
          theme: "system",
          autoSyncMinutes: 5,
          recentVaults: ["/vaults/work"],
        };
      case "notes_open_vault":
        return { path: "/vaults/work", name: "work" };
      case "notes_list_entries":
        return [{ path: "ideas.md", kind: "note" }];
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
