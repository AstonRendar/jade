import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { resetStores } from "../../shared/testing/resetStores";
import { AUTOSAVE_DELAY_MS, useEditor } from "./editorStore";

type Call = { command: string; args: Record<string, unknown> };

describe("editor store", () => {
  let calls: Call[];
  let disk: Record<string, string>;

  beforeEach(() => {
    resetStores();
    calls = [];
    disk = { "ideas.md": "first", "plan.md": "plan" };
    mockIPC((command, args) => {
      const values = (args ?? {}) as Record<string, unknown>;
      calls.push({ command, args: values });
      if (command === "notes_read_note") {
        return disk[values.path as string];
      }
      if (command === "notes_save_note") {
        disk[values.path as string] = values.content as string;
      }
      return null;
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    clearMocks();
  });

  it("loads the note from disk", async () => {
    // Arrange
    const editor = useEditor.getState();

    // Act
    await editor.load("ideas.md");

    // Assert
    expect(useEditor.getState()).toMatchObject({
      path: "ideas.md",
      draft: "first",
    });
  });

  it("saves automatically shortly after the last edit", async () => {
    // Arrange
    vi.useFakeTimers();
    await useEditor.getState().load("ideas.md");

    // Act
    useEditor.getState().edit("second");
    useEditor.getState().edit("third");
    await vi.advanceTimersByTimeAsync(AUTOSAVE_DELAY_MS);

    // Assert
    expect(saves()).toEqual([{ path: "ideas.md", content: "third" }]);
  });

  it("flushes pending edits immediately", async () => {
    // Arrange
    await useEditor.getState().load("ideas.md");
    useEditor.getState().edit("second");

    // Act
    await useEditor.getState().flush();

    // Assert
    expect(saves()).toEqual([{ path: "ideas.md", content: "second" }]);
    expect(useEditor.getState().isDirty()).toBe(false);
  });

  it("saves the previous note before opening another", async () => {
    // Arrange
    await useEditor.getState().load("ideas.md");
    useEditor.getState().edit("second");

    // Act
    await useEditor.getState().load("plan.md");

    // Assert
    expect(saves()).toEqual([{ path: "ideas.md", content: "second" }]);
    expect(useEditor.getState().draft).toBe("plan");
  });

  it("reloads the open note when it changes on disk and has no edits", async () => {
    // Arrange
    await useEditor.getState().load("ideas.md");
    disk["ideas.md"] = "from another device";

    // Act
    await useEditor.getState().reloadIfChanged(["ideas.md"]);

    // Assert
    expect(useEditor.getState().draft).toBe("from another device");
  });

  it("keeps unsaved edits when the note changes on disk", async () => {
    // Arrange
    await useEditor.getState().load("ideas.md");
    useEditor.getState().edit("mine");
    disk["ideas.md"] = "theirs";

    // Act
    await useEditor.getState().reloadIfChanged(["ideas.md"]);

    // Assert
    expect(useEditor.getState().draft).toBe("mine");
  });

  it("saves and acknowledges when the core asks to flush", async () => {
    // Arrange
    await useEditor.getState().load("ideas.md");
    useEditor.getState().edit("second");

    // Act
    await useEditor.getState().answerFlushRequest(7);

    // Assert
    expect(commands()).toEqual([
      "notes_read_note",
      "notes_save_note",
      "editor_flush_completed",
    ]);
    expect(calls.at(-1)?.args).toEqual({ id: 7 });
  });

  function saves() {
    return calls
      .filter((call) => call.command === "notes_save_note")
      .map((call) => call.args);
  }

  function commands() {
    return calls.map((call) => call.command);
  }
});
