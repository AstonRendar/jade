import { clearMocks, mockConvertFileSrc, mockIPC } from "@tauri-apps/api/mocks";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider } from "react-i18next";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createI18n } from "../../shared/i18n";
import { resetStores } from "../../shared/testing/resetStores";
import { NoteView } from "../editor/NoteView";
import { Preview } from "../editor/Preview";
import { useEditor } from "../store/editorStore";
import { useImages } from "../store/imageStore";
import { useVault } from "../store/vaultStore";
import { ImageView } from "./ImageView";

describe("images", () => {
  let resolved: string[][];
  let saved: number[];

  beforeEach(() => {
    resetStores();
    resolved = [];
    saved = [];
    mockConvertFileSrc("macos");
    mockIPC(
      (command, args) => {
        if (command === "notes_resolve_files") {
          const targets = (args as { targets: string[] }).targets;
          resolved.push(targets);
          return targets.map((target) => ({
            target,
            path: target.startsWith("missing") ? null : `Recursos/${target}`,
          }));
        }
        if (command === "notes_save_attachment") {
          saved.push((args as unknown as Uint8Array).length);
          return { path: "Recursos/foto.png", link: "foto.png" };
        }
        if (command === "notes_read_note") {
          return "# Torre\n";
        }
        if (command === "notes_read_properties") {
          return { properties: [], bodyStart: 0, valid: true };
        }
        if (
          command === "notes_list_properties" ||
          command === "notes_list_entries"
        ) {
          return [];
        }
        return null;
      },
      { shouldMockEvents: true },
    );
    useVault.setState({ vault: { path: "/vaults/torres", name: "torres" } });
  });

  afterEach(async () => {
    cleanup();
    await new Promise((resolve) => setTimeout(resolve, 0));
    clearMocks();
  });

  async function renderWithI18n(node: React.ReactNode) {
    const i18n = await createI18n("en");
    render(<I18nextProvider i18n={i18n}>{node}</I18nextProvider>);
  }

  it("shows embedded images in the preview and enlarges them on click", async () => {
    // Arrange
    await renderWithI18n(
      <Preview
        source={"![[mapa.png|300]]\n\n![[missing.png]]"}
        onFollow={() => {}}
      />,
    );

    // Act
    const image = await waitFor(() => {
      const found = document.querySelector("img.embed-image");
      expect(found).not.toBeNull();
      return found as HTMLImageElement;
    });
    await userEvent.click(image);

    // Assert
    expect(image).toHaveAttribute(
      "src",
      expect.stringContaining("Recursos%2Fmapa.png"),
    );
    expect(image).toHaveAttribute("width", "300");
    expect(screen.getByText("missing.png")).toBeInTheDocument();
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("resolves each image once and forgets them when asked", async () => {
    // Arrange
    const images = useImages.getState();

    // Act
    await images.resolve(["a.png", "a.png", "https://example.com/b.png"]);
    await images.resolve(["a.png"]);
    useImages.getState().forget();
    await useImages.getState().resolve(["a.png"]);

    // Assert
    expect(resolved).toEqual([["a.png"], ["a.png"]]);
    expect(useImages.getState().urls["a.png"]).toContain("Recursos%2Fa.png");
  });

  it("resolves the known images again when the vault changes on disk", async () => {
    // Arrange
    await useImages.getState().resolve(["a.png", "missing.png"]);

    // Act
    await useImages.getState().refresh();

    // Assert
    expect(resolved).toEqual([
      ["a.png", "missing.png"],
      ["a.png", "missing.png"],
    ]);
    expect(useImages.getState().urls["a.png"]).toContain("a.png");
    expect(useImages.getState().urls["missing.png"]).toBeNull();
  });

  it("does not resolve images without an open vault", async () => {
    // Arrange
    useVault.setState({ vault: null });

    // Act
    await useImages.getState().resolve(["a.png"]);

    // Assert
    expect(resolved).toEqual([]);
  });

  it("inserts a picked image into the note", async () => {
    // Arrange
    await useEditor.getState().load("Lore/Torre.md");
    await renderWithI18n(<NoteView />);
    await waitFor(() =>
      expect(document.querySelector(".cm-content")).not.toBeNull(),
    );
    const picker = screen.getByTestId("image-picker") as HTMLInputElement;
    const file = new File([new Uint8Array([1, 2, 3])], "image.png", {
      type: "image/png",
    });

    // Act
    await userEvent.click(screen.getByRole("button", { name: "Insert image" }));
    fireEvent.change(picker, { target: { files: [file] } });

    // Assert
    await waitFor(() =>
      expect(useEditor.getState().draft).toContain("![[foto.png]]"),
    );
    expect(saved).toEqual([3]);
  });

  it("opens image files in a viewer", async () => {
    // Arrange
    await renderWithI18n(<ImageView path="Recursos/mapa.png" />);

    // Act
    const image = await screen.findByRole("img", { name: "mapa.png" });

    // Assert
    expect(
      screen.getByRole("heading", { name: "mapa.png" }),
    ).toBeInTheDocument();
    expect(image).toHaveAttribute("src", expect.stringContaining("mapa.png"));
  });
});
