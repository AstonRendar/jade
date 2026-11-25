import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { resetStores } from "../../shared/testing/resetStores";
import { useVault } from "./vaultStore";

describe("vault tabs", () => {
  beforeEach(() => {
    resetStores();
    mockIPC((command) => (command === "notes_list_entries" ? [] : null));
    useVault.setState({ vault: { path: "/vaults/work", name: "work" } });
  });

  afterEach(() => {
    clearMocks();
  });

  function state() {
    const { tabs, selected } = useVault.getState();
    return { tabs, selected };
  }

  it("opens a note in the current tab and an open note by switching to it", () => {
    // Arrange
    const vault = useVault.getState();

    // Act
    vault.select("a.md");
    vault.select("b.md");
    vault.openInNewTab("c.md");
    vault.select("b.md");

    // Assert
    expect(state()).toEqual({ tabs: ["b.md", "c.md"], selected: "b.md" });
  });

  it("opens new tabs next to the current one", () => {
    // Arrange
    const vault = useVault.getState();
    vault.select("a.md");
    vault.openInNewTab("b.md");
    vault.select("a.md");

    // Act
    vault.openInNewTab("c.md");
    vault.openInNewTab("c.md");

    // Assert
    expect(state()).toEqual({
      tabs: ["a.md", "c.md", "b.md"],
      selected: "c.md",
    });
  });

  it("closes tabs and activates the neighbour of the closed active tab", () => {
    // Arrange
    const vault = useVault.getState();
    vault.select("a.md");
    vault.openInNewTab("b.md");
    vault.openInNewTab("c.md");

    // Act
    vault.closeTab("a.md");
    const afterInactive = state();
    vault.closeTab("c.md");
    const afterLast = state();
    vault.closeTab("b.md");

    // Assert
    expect(afterInactive).toEqual({ tabs: ["b.md", "c.md"], selected: "c.md" });
    expect(afterLast).toEqual({ tabs: ["b.md"], selected: "b.md" });
    expect(state()).toEqual({ tabs: [], selected: null });
  });

  it("cycles through the tabs in both directions", () => {
    // Arrange
    const vault = useVault.getState();
    vault.cycleTab(1);
    vault.select("a.md");
    vault.openInNewTab("b.md");

    // Act
    vault.cycleTab(1);
    const forward = state().selected;
    vault.cycleTab(-1);

    // Assert
    expect(forward).toBe("a.md");
    expect(state().selected).toBe("b.md");
  });

  it("follows renamed notes and folders and drops deleted ones", async () => {
    // Arrange
    const vault = useVault.getState();
    vault.select("notes/a.md");
    vault.openInNewTab("notes/b.md");
    vault.openInNewTab("c.md");
    vault.select("notes/b.md");

    // Act
    await vault.rename("notes", "archive");
    const renamed = state();
    await vault.remove("archive");

    // Assert
    expect(renamed).toEqual({
      tabs: ["archive/a.md", "archive/b.md", "c.md"],
      selected: "archive/b.md",
    });
    expect(state()).toEqual({ tabs: ["c.md"], selected: "c.md" });
  });

  it("forgets the tabs when the vault closes", () => {
    // Arrange
    useVault.getState().select("a.md");

    // Act
    useVault.getState().close();

    // Assert
    expect(state()).toEqual({ tabs: [], selected: null });
  });
});
