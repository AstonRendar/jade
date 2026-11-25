import { emit } from "@tauri-apps/api/event";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useVault } from "../notes/store/vaultStore";
import type { PreferencesView } from "../shared/bindings/PreferencesView";
import type { RepositoryStatusView } from "../shared/bindings/RepositoryStatusView";
import { createI18n } from "../shared/i18n";
import { resetStores } from "../shared/testing/resetStores";
import { Workspace } from "./Workspace";

type Call = { command: string; args: Record<string, unknown> };

describe("navigation, history, sync and settings", () => {
  let calls: Call[];
  let preferences: PreferencesView;
  let status: RepositoryStatusView;

  beforeEach(() => {
    resetStores();
    calls = [];
    preferences = {
      locale: "en",
      theme: "system",
      autoSyncMinutes: 5,
      recentVaults: ["/vaults/work"],
    };
    status = {
      isRepository: true,
      branch: "main",
      remote: "https://git.example.com/work.git",
      pendingChanges: 2,
      ahead: 0,
      behind: 0,
    };
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

  it("opens a note from the quick switcher", async () => {
    // Arrange
    await openVault();

    // Act
    await userEvent.keyboard("{Meta>}o{/Meta}");
    await userEvent.type(screen.getByRole("combobox"), "pla{Enter}");

    // Assert
    await waitFor(() =>
      expect(useVault.getState().selected).toBe("projects/plan.md"),
    );
  });

  it("opens a base from the tree as a table and a note from the base", async () => {
    // Arrange
    await openVault();

    // Act
    await userEvent.click(screen.getByRole("treeitem", { name: "Index.base" }));
    await userEvent.click(
      await screen.findByRole("button", { name: "ideas.md" }),
    );

    // Assert
    expect(called("notes_read_note")).toEqual([{ path: "ideas.md" }]);
    await screen.findByRole("heading", { level: 1, name: "ideas" });
  });

  it("keeps several notes open in tabs", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("treeitem", { name: "ideas" }));

    // Act
    await userEvent.click(screen.getByRole("treeitem", { name: "projects" }));
    fireEvent.click(await screen.findByRole("treeitem", { name: "plan" }), {
      metaKey: true,
    });

    // Assert
    const tabs = within(screen.getByRole("tablist", { name: "Open notes" }))
      .getAllByRole("tab")
      .map((tab) => tab.textContent);
    expect(tabs).toEqual(["ideas", "plan"]);
    expect(screen.getByRole("tab", { name: "plan" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await screen.findByRole("heading", { level: 1, name: "plan" });
  });

  it("switches, cycles and closes tabs", async () => {
    // Arrange
    await openVault();
    useVault.getState().select("ideas.md");
    useVault.getState().openInNewTab("projects/plan.md");
    await screen.findByRole("tab", { name: "plan" });

    // Act
    await userEvent.click(screen.getByRole("tab", { name: "ideas" }));
    const clicked = useVault.getState().selected;
    await userEvent.keyboard("{Control>}{Tab}{/Control}");
    const cycled = useVault.getState().selected;
    await userEvent.click(screen.getByRole("button", { name: "Close plan" }));
    await userEvent.keyboard("{Meta>}w{/Meta}");

    // Assert
    expect(clicked).toBe("ideas.md");
    expect(cycled).toBe("projects/plan.md");
    expect(
      screen.queryByRole("tablist", { name: "Open notes" }),
    ).not.toBeInTheDocument();
    expect(useVault.getState().selected).toBeNull();
  });

  it("opens a note in a new tab from its context menu", async () => {
    // Arrange
    await openVault();
    useVault.getState().select("ideas.md");
    await userEvent.click(screen.getByRole("treeitem", { name: "projects" }));

    // Act
    fireEvent.contextMenu(
      await screen.findByRole("treeitem", { name: "plan" }),
    );
    await userEvent.click(
      screen.getByRole("menuitem", { name: "Open in new tab" }),
    );

    // Assert
    expect(useVault.getState().tabs).toEqual(["ideas.md", "projects/plan.md"]);
  });

  it("finds text in every note and opens a hit", async () => {
    // Arrange
    await openVault();

    // Act
    await userEvent.keyboard("{Meta>}{Shift>}f{/Shift}{/Meta}");
    await userEvent.type(
      screen.getByRole("searchbox", { name: "Search files and text…" }),
      "jade",
    );
    await userEvent.click(
      await screen.findByRole("button", { name: /Notes about jade/ }),
    );

    // Assert
    await waitFor(() => expect(useVault.getState().selected).toBe("ideas.md"));
    expect(called("notes_search")).toContainEqual({ query: "jade" });
  });

  it("finds files by name and opens them, also in a new tab", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Search" }));
    const box = screen.getByRole("searchbox", {
      name: "Search files and text…",
    });

    // Act
    await userEvent.type(box, "pln");
    const files = screen
      .getAllByRole("button", { name: /plan/ })
      .map((button) => button.textContent);
    await userEvent.keyboard("{Enter}");
    const opened = useVault.getState().selected;
    await userEvent.clear(box);
    await userEvent.type(box, "idea");
    fireEvent.click(
      screen.getAllByRole("button", { name: /^ideas/ })[0] as HTMLElement,
      { metaKey: true },
    );

    // Assert
    expect(files[0]).toBe("planprojects");
    expect(opened).toBe("projects/plan.md");
    expect(useVault.getState().tabs).toEqual(["projects/plan.md", "ideas.md"]);
  });

  it("goes back to the tree from the search panel", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Search" }));

    // Act
    const hint = screen.getByText(/Type to find files/);
    await userEvent.keyboard("{Escape}");

    // Assert
    expect(hint).not.toBeInTheDocument();
    expect(screen.getByRole("treeitem", { name: "ideas" })).toBeInTheDocument();
  });

  it("lists the backlinks of the open note", async () => {
    // Arrange
    await openNote("projects/plan.md");

    // Act
    const panel = await screen.findByRole("tabpanel", { name: "Backlinks" });
    await userEvent.click(
      within(panel).getByRole("button", { name: /see \[\[plan\]\]/ }),
    );

    // Assert
    await waitFor(() => expect(useVault.getState().selected).toBe("ideas.md"));
  });

  it("restores an earlier version of the open note", async () => {
    // Arrange
    await openNote("projects/plan.md");
    await userEvent.click(screen.getByRole("tab", { name: "History" }));

    // Act
    await userEvent.click(
      await screen.findByRole("button", { name: /First draft/ }),
    );
    expect(await screen.findByText("old content")).toBeInTheDocument();
    await userEvent.click(
      screen.getByRole("button", { name: "Restore this version" }),
    );

    // Assert
    await waitFor(() =>
      expect(called("notes_save_note")).toContainEqual({
        path: "projects/plan.md",
        content: "old content",
      }),
    );
  });

  it("synchronizes on demand and reports conflict copies", async () => {
    // Arrange
    await openVault();

    // Act
    await userEvent.click(screen.getByRole("button", { name: "Sync now" }));

    // Assert
    expect(
      await screen.findByText(
        "Both devices changed “ideas.md”. The other version was saved as “ideas (conflict 2026-11-22 1030).md”.",
      ),
    ).toBeInTheDocument();
    expect(called("sync_synchronize")).toHaveLength(1);
  });

  it("shows the repository state", async () => {
    // Arrange
    // Act
    await openVault();

    // Assert
    expect(await screen.findByText("2 pending")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "⎇ main" })).toBeInTheDocument();
  });

  it("switches to another branch and reloads the vault", async () => {
    // Arrange
    await openVault();
    await userEvent.click(
      await screen.findByRole("button", { name: "⎇ main" }),
    );

    // Act
    await userEvent.click(
      await screen.findByRole("option", { name: /develop/ }),
    );

    // Assert
    await waitFor(() =>
      expect(called("sync_switch_branch")).toEqual([{ branch: "develop" }]),
    );
    await waitFor(() =>
      expect(called("notes_list_entries").length).toBeGreaterThan(1),
    );
  });

  it("closes the branch list when the current branch is chosen", async () => {
    // Arrange
    await openVault();
    await userEvent.click(
      await screen.findByRole("button", { name: "⎇ main" }),
    );

    // Act
    await userEvent.click(await screen.findByRole("option", { name: /main/ }));

    // Assert
    expect(called("sync_switch_branch")).toEqual([]);
    expect(screen.queryByRole("option")).not.toBeInTheDocument();
  });

  it("offers to version a folder that is not a repository", async () => {
    // Arrange
    status = { ...status, isRepository: false, branch: null, remote: null };
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Settings" }));

    // Act
    await userEvent.click(
      await screen.findByRole("button", { name: "Version this folder" }),
    );

    // Assert
    await waitFor(() => expect(called("sync_init_repository")).toHaveLength(1));
  });

  it("saves the chosen language, theme and sync interval", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Settings" }));
    const dialog = await screen.findByRole("dialog", { name: "Settings" });

    // Act
    await userEvent.selectOptions(
      within(dialog).getByLabelText("Language"),
      "es",
    );
    await userEvent.selectOptions(
      within(dialog).getByLabelText("Theme"),
      "dark",
    );
    const minutes = within(dialog).getByLabelText(
      "Sync every (minutes, 0 = off)",
    );
    await userEvent.clear(minutes);
    await userEvent.type(minutes, "15");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));

    // Assert
    await waitFor(() =>
      expect(called("preferences_update")).toContainEqual({
        locale: "es",
        theme: "dark",
        autoSyncMinutes: 15,
      }),
    );
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("dark"),
    );
  });

  it("stores the access token and the remote", async () => {
    // Arrange
    await openVault();
    await userEvent.click(screen.getByRole("button", { name: "Settings" }));
    const dialog = await screen.findByRole("dialog", { name: "Settings" });

    // Act
    await userEvent.type(
      within(dialog).getByLabelText("GitHub token"),
      "token-123",
    );
    const remote = within(dialog).getByLabelText("Remote repository");
    await userEvent.clear(remote);
    await userEvent.type(remote, "https://git.example.com/new.git");
    await userEvent.click(within(dialog).getByRole("button", { name: "Save" }));

    // Assert
    await waitFor(() =>
      expect(called("sync_store_access_token")).toContainEqual({
        token: "token-123",
      }),
    );
    expect(called("sync_connect_remote")).toContainEqual({
      url: "https://git.example.com/new.git",
    });
  });

  async function openVault() {
    const i18n = await createI18n("en");
    render(<Workspace i18n={i18n} />);
    await userEvent.click(await screen.findByRole("button", { name: /work/ }));
    await screen.findByRole("treeitem", { name: "ideas" });
  }

  async function openNote(path: string) {
    await openVault();
    useVault.getState().select(path);
    await screen.findByRole("heading", { level: 1, name: "plan" });
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
      case "preferences_update":
        preferences = {
          ...preferences,
          locale: args.locale as string | null,
          theme: args.theme as string,
          autoSyncMinutes: args.autoSyncMinutes as number,
        };
        return preferences;
      case "notes_open_vault":
        return { path: "/vaults/work", name: "work" };
      case "notes_list_entries":
        return [
          { path: "ideas.md", kind: "note" },
          { path: "projects", kind: "folder" },
          { path: "projects/plan.md", kind: "note" },
          { path: "Index.base", kind: "file" },
        ];
      case "sync_list_branches":
        return { current: "main", branches: ["develop", "main"] };
      case "notes_read_properties":
        return { properties: [], bodyStart: 0, valid: true };
      case "notes_list_properties":
        return [];
      case "notes_read_note":
        return args.path === "ideas.md"
          ? "Notes about jade"
          : "current content";
      case "notes_query_base":
        return {
          views: [{ name: "Table", kind: "table" }],
          view: 0,
          columns: [{ key: "file.name", displayName: null, width: null }],
          rows: [
            {
              path: "ideas.md",
              cells: [{ type: "text", value: "ideas.md" }],
            },
          ],
          total: 1,
        };
      case "notes_search":
        return [{ path: "ideas.md", line: 1, text: "Notes about jade" }];
      case "notes_find_backlinks":
        return [{ source: "ideas.md", line: 3, text: "see [[plan]]" }];
      case "sync_get_status":
        return status;
      case "sync_has_access_token":
        return false;
      case "sync_list_note_history":
        return [
          {
            id: "b".repeat(40),
            summary: "Sync from laptop",
            author: "Example",
            epochSeconds: 1795000000,
          },
          {
            id: "a".repeat(40),
            summary: "First draft",
            author: "Example",
            epochSeconds: 1794000000,
          },
        ];
      case "sync_read_note_revision":
        return "old content";
      case "sync_synchronize":
        void emit("sync://finished", report());
        return report();
      default:
        return null;
    }
  }

  function report() {
    return {
      committed: true,
      changedPaths: ["ideas (conflict 2026-11-22 1030).md"],
      conflictCopies: [
        { original: "ideas.md", copy: "ideas (conflict 2026-11-22 1030).md" },
      ],
      pushed: true,
    };
  }
});
