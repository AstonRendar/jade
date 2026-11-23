import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider } from "react-i18next";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { BaseTableView } from "../../shared/bindings/BaseTableView";
import type { CellView } from "../../shared/bindings/CellView";
import { createI18n } from "../../shared/i18n";
import { resetStores } from "../../shared/testing/resetStores";
import { useVault } from "../store/vaultStore";
import { BaseView } from "./BaseView";
import { compareCells } from "./compareCells";

const TABLE: BaseTableView = {
  views: [
    { name: "Characters", kind: "table" },
    { name: "Everything", kind: "table" },
  ],
  view: 0,
  columns: [
    { key: "file.name", displayName: null, width: 200 },
    { key: "tipo", displayName: "Type", width: null },
    { key: "note.edad", displayName: null, width: null },
    { key: "file.tags", displayName: null, width: null },
    { key: "mentor", displayName: null, width: null },
    { key: "activo", displayName: null, width: null },
    { key: "file.mtime", displayName: null, width: null },
    { key: "formula.self", displayName: null, width: null },
  ],
  rows: [
    {
      path: "Characters/Ana.md",
      cells: [
        { type: "text", value: "Ana.md" },
        { type: "text", value: "personaje" },
        { type: "number", value: 42 },
        { type: "list", items: [{ type: "text", value: "saga" }] },
        { type: "link", target: "Bruno", label: null },
        { type: "boolean", value: true },
        { type: "date", milliseconds: 1767225600000 },
        { type: "file", path: "Characters/Ana.md" },
      ],
    },
    {
      path: "Places/Tower.md",
      cells: [
        { type: "text", value: "Tower.md" },
        { type: "text", value: "lugar" },
        { type: "null" },
        { type: "list", items: [] },
        { type: "null" },
        { type: "boolean", value: false },
        { type: "date", milliseconds: 1767225600000 },
        { type: "file", path: "Places/Tower.md" },
      ],
    },
  ],
  total: 2,
};

describe("BaseView", () => {
  let queries: Array<string | null>;
  let followed: string[];
  let failing: boolean;

  beforeEach(() => {
    resetStores();
    queries = [];
    followed = [];
    failing = false;
    mockIPC(
      (command, args) => {
        const values = (args ?? {}) as Record<string, unknown>;
        if (command === "notes_query_base") {
          queries.push(values.view as string | null);
          if (failing) {
            throw { code: "invalid_base", params: { detail: "views" } };
          }
          return { ...TABLE, view: values.view === "Everything" ? 1 : 0 };
        }
        if (command === "notes_follow_link") {
          followed.push(values.target as string);
          return { path: "Characters/Bruno.md", created: false };
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

  it("shows the base as a table with labelled columns and formatted cells", async () => {
    // Arrange
    await renderBase();

    // Act
    const table = await screen.findByRole("table");

    // Assert
    const headers = within(table)
      .getAllByRole("columnheader")
      .map((header) => header.textContent);
    expect(headers).toEqual([
      "File name",
      "Type",
      "edad",
      "Tags",
      "mentor",
      "activo",
      "Modified",
      "self",
    ]);
    expect(screen.getByRole("heading", { name: "Index" })).toBeInTheDocument();
    expect(screen.getByText("2 results")).toBeInTheDocument();
    expect(screen.getByText("saga")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
    expect(screen.getAllByRole("checkbox")[0]).toBeChecked();
  });

  it("opens notes from the file columns and follows links", async () => {
    // Arrange
    await renderBase();
    await screen.findByRole("table");

    // Act
    await userEvent.click(
      screen.getAllByRole("button", { name: "Tower.md" })[0] as HTMLElement,
    );
    const openedFromName = useVault.getState().selected;
    await userEvent.click(
      screen.getAllByRole("button", { name: "Ana.md" })[1] as HTMLElement,
    );
    const openedFromFile = useVault.getState().selected;
    await userEvent.click(screen.getByRole("button", { name: "Bruno" }));

    // Assert
    expect(openedFromName).toBe("Places/Tower.md");
    expect(openedFromFile).toBe("Characters/Ana.md");
    await waitFor(() => expect(followed).toEqual(["Bruno"]));
  });

  it("sorts by a column ascending, descending and back", async () => {
    // Arrange
    await renderBase();
    await screen.findByRole("table");
    const typeHeader = screen.getByRole("button", { name: "Type" });

    // Act
    await userEvent.click(typeHeader);
    const ascending = firstColumn();
    await userEvent.click(typeHeader);
    const descending = firstColumn();
    await userEvent.click(typeHeader);
    const unsorted = firstColumn();

    // Assert
    expect(ascending).toEqual(["Tower.md", "Ana.md"]);
    expect(descending).toEqual(["Ana.md", "Tower.md"]);
    expect(unsorted).toEqual(["Ana.md", "Tower.md"]);
  });

  it("switches views and reloads when the vault changes", async () => {
    // Arrange
    await renderBase();
    await screen.findByRole("table");

    // Act
    await userEvent.click(screen.getByRole("tab", { name: "Everything" }));
    await waitFor(() =>
      expect(screen.getByRole("tab", { name: "Everything" })).toHaveAttribute(
        "aria-selected",
        "true",
      ),
    );
    await emit("notes://changed", { paths: ["Characters/Ana.md"] });

    // Assert
    await waitFor(() =>
      expect(queries).toEqual([null, "Everything", "Everything"]),
    );
  });

  it("explains why a base cannot be shown", async () => {
    // Arrange
    failing = true;

    // Act
    await renderBase();

    // Assert
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "This base is not valid: views",
    );
  });

  it("orders cells of every type with empty values last", () => {
    // Arrange
    const cells = [
      { type: "null" },
      { type: "number", value: 10 },
      { type: "number", value: 9 },
      { type: "text", value: "b" },
      { type: "list", items: [] },
      { type: "link", target: "a", label: null },
      { type: "file", path: "c" },
      { type: "boolean", value: true },
      { type: "date", milliseconds: 5 },
      { type: "list", items: [{ type: "text", value: "a2" }] },
    ] satisfies CellView[];

    // Act
    const sorted = [...(cells as CellView[])]
      .sort(compareCells)
      .map((cell) => cell.type);

    // Assert
    expect(sorted.slice(0, 3)).toEqual(["boolean", "date", "number"]);
    expect(sorted.slice(-2)).toEqual(["null", "list"]);
    expect(compareCells(undefined, { type: "null" })).toBe(0);
  });

  async function renderBase() {
    const i18n = await createI18n("en");
    useVault.setState({
      vault: { path: "/vaults/work", name: "work" },
      selected: "Index.base",
    });
    render(
      <I18nextProvider i18n={i18n}>
        <BaseView path="Index.base" />
      </I18nextProvider>,
    );
  }

  function firstColumn() {
    return screen
      .getAllByRole("row")
      .slice(1)
      .map((row) => within(row).getAllByRole("cell")[0]?.textContent);
  }
});
