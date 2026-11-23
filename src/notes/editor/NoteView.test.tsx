import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider } from "react-i18next";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createI18n } from "../../shared/i18n";
import { resetStores } from "../../shared/testing/resetStores";
import { useEditor } from "../store/editorStore";
import { useVault } from "../store/vaultStore";
import type { NotePropertiesView } from "../../shared/bindings/NotePropertiesView";
import type { PropertySummaryView } from "../../shared/bindings/PropertySummaryView";
import type { PropertyView } from "../../shared/bindings/PropertyView";
import { NoteView } from "./NoteView";

const HEAD = "---\ntipo: personaje\n---\n";
const NOTES: Record<string, string> = {
  "ideas.md": "# Ideas\n\nSee [[Plan]].",
  "ana.md": `${HEAD}Ana's story`,
  "broken.md": "---\n: [\n---\nbody",
};
const PROPERTIES: PropertyView[] = [
  { key: "tipo", value: { type: "text", value: "personaje" } },
  {
    key: "tags",
    value: { type: "list", items: [{ type: "text", value: "saga" }] },
  },
  { key: "edad", value: { type: "number", value: 42 } },
  { key: "activo", value: { type: "boolean", value: true } },
  { key: "nacido", value: { type: "text", value: "1024-03-04" } },
  { key: "mentor", value: { type: "text", value: "[[Bruno|El maestro]]" } },
  {
    key: "datos",
    value: {
      type: "map",
      entries: [{ key: "nivel", value: { type: "number", value: 3 } }],
    },
  },
  { key: "vacío", value: { type: "null" } },
];
const CATALOG: PropertySummaryView[] = [
  { name: "tipo", kind: "text", count: 2, values: ["lugar", "personaje"] },
  { name: "tags", kind: "tags", count: 1, values: ["saga"] },
  { name: "nacido", kind: "date", count: 1, values: [] },
  { name: "edad", kind: "number", count: 1, values: [] },
  { name: "aliases", kind: "aliases", count: 1, values: [] },
];

function bodyOf(content: string): string {
  return content.startsWith("---\n")
    ? content.slice(content.indexOf("---\n", 4) + 4)
    : content;
}

function readProperties(content: string): NotePropertiesView {
  if (content.startsWith("---\n: [")) {
    return { properties: [], bodyStart: 0, valid: false };
  }
  if (!content.startsWith("---\n")) {
    return { properties: [], bodyStart: 0, valid: true };
  }
  return {
    properties: content.startsWith(HEAD) ? PROPERTIES : [],
    bodyStart: content.length - bodyOf(content).length,
    valid: true,
  };
}

describe("NoteView", () => {
  let followed: string[];
  let written: PropertyView[][];

  beforeEach(() => {
    resetStores();
    followed = [];
    written = [];
    mockIPC(
      (command, args) => {
        const values = (args ?? {}) as Record<string, unknown>;
        if (command === "notes_read_note") {
          return NOTES[values.path as string] ?? "plan";
        }
        if (command === "notes_read_properties") {
          return readProperties(values.content as string);
        }
        if (command === "notes_write_properties") {
          written.push(values.properties as PropertyView[]);
          return `${HEAD}${bodyOf(values.content as string)}`;
        }
        if (command === "notes_list_properties") {
          return CATALOG;
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

  it("shows the properties as fields when reading and the body alone", async () => {
    // Arrange
    await renderNote("ana.md");

    // Act
    await userEvent.click(screen.getByRole("button", { name: "Preview" }));

    // Assert
    const panel = screen.getByRole("region", { name: "Properties" });
    expect(panel).toHaveTextContent("personaje");
    expect(panel).toHaveTextContent("saga");
    expect(panel).toHaveTextContent("42");
    expect(panel).toHaveTextContent("nivel: 3");
    expect(screen.getByRole("checkbox")).toBeChecked();
    expect(screen.queryByText(/tipo: personaje/)).not.toBeInTheDocument();
    expect(screen.getByText("Ana's story")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "El maestro" }));
    expect(followed).toEqual(["Bruno"]);
  });

  it("edits the body without touching the properties block", async () => {
    // Arrange
    await renderNote("ana.md");

    // Act
    const editor = document.querySelector(".cm-content") as HTMLElement;
    const shown = editor.textContent;

    // Assert
    expect(shown).toBe("Ana's story");
    expect(screen.getByRole("combobox", { name: "tipo" })).toHaveValue(
      "personaje",
    );
  });

  it("writes changed, added and removed properties back into the note", async () => {
    // Arrange
    await renderNote("ana.md");
    const tipo = screen.getByRole("combobox", { name: "tipo" });

    // Act
    await userEvent.clear(tipo);
    await userEvent.type(tipo, "lugar{Enter}");
    await userEvent.click(screen.getByRole("checkbox", { name: "activo" }));
    await userEvent.click(screen.getByRole("button", { name: "Remove saga" }));
    await userEvent.type(
      screen.getByRole("combobox", { name: "Add a value to tags" }),
      "norte{Enter}",
    );
    await userEvent.click(screen.getByRole("button", { name: "Remove vacío" }));
    await userEvent.click(screen.getByRole("button", { name: /Add property/ }));
    await userEvent.type(
      screen.getByRole("combobox", { name: "Property name" }),
      "aliases{Enter}",
    );

    // Assert
    await waitFor(() => expect(written).toHaveLength(6));
    expect(written[0]?.[0]).toEqual({
      key: "tipo",
      value: { type: "text", value: "lugar" },
    });
    expect(written[1]?.[3]).toEqual({
      key: "activo",
      value: { type: "boolean", value: false },
    });
    expect(written[2]?.[1]?.value).toEqual({ type: "list", items: [] });
    expect(written[3]?.[1]?.value).toEqual({
      type: "list",
      items: [
        { type: "text", value: "saga" },
        { type: "text", value: "norte" },
      ],
    });
    expect(written[4]?.map((property) => property.key)).not.toContain("vacío");
    expect(written[5]?.at(-1)).toEqual({
      key: "aliases",
      value: { type: "list", items: [] },
    });
    expect(useEditor.getState().draft).toBe(`${HEAD}Ana's story`);
  });

  it("renames properties and converts numbers", async () => {
    // Arrange
    await renderNote("ana.md");

    // Act
    const name = screen.getAllByRole("textbox", {
      name: "Property name",
    })[0] as HTMLElement;
    await userEvent.clear(name);
    await userEvent.type(name, "clase{Enter}");
    const age = screen.getByRole("spinbutton", { name: "edad" });
    await userEvent.clear(age);
    await userEvent.type(age, "43{Enter}");
    const born = screen.getByLabelText("nacido");
    await userEvent.clear(born);

    // Assert
    await waitFor(() => expect(written).toHaveLength(3));
    expect(written[0]?.[0]?.key).toBe("clase");
    expect(written[1]?.[2]).toEqual({
      key: "edad",
      value: { type: "number", value: 43 },
    });
    expect(written[2]?.[4]).toEqual({ key: "nacido", value: { type: "null" } });
  });

  it("warns about a malformed properties block and keeps it as text", async () => {
    // Arrange
    await renderNote("broken.md");

    // Act
    const alert = screen.getByRole("alert");

    // Assert
    expect(alert).toHaveTextContent("not valid YAML");
    expect(document.querySelector(".cm-content")?.textContent).toContain(": [");
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
    await waitFor(() =>
      expect(document.querySelector(".cm-content")).not.toBeNull(),
    );
  }
});
