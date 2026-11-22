import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ChangedPathsView } from "../../shared/bindings/ChangedPathsView";
import type { EntryView } from "../../shared/bindings/EntryView";
import type { VaultView } from "../../shared/bindings/VaultView";
import { call } from "../../shared/gateway/call";

export const notesGateway = {
  openVault: (path: string) => call<VaultView>("notes_open_vault", { path }),
  listEntries: () => call<EntryView[]>("notes_list_entries"),
  createNote: (folder: string | null, title: string) =>
    call<string>("notes_create_note", { folder, title }),
  createFolder: (path: string) => call<string>("notes_create_folder", { path }),
  renameEntry: (from: string, to: string) =>
    call<string>("notes_rename_entry", { from, to }),
  deleteEntry: (path: string) => call<null>("notes_delete_entry", { path }),
  onChanged: (handler: (paths: string[]) => void): Promise<UnlistenFn> =>
    listen<ChangedPathsView>("notes://changed", (event) =>
      handler(event.payload.paths),
    ),
};
