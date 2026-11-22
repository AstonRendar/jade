import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ChangedPathsView } from "../../shared/bindings/ChangedPathsView";
import type { EntryView } from "../../shared/bindings/EntryView";
import type { FlushRequestView } from "../../shared/bindings/FlushRequestView";
import type { BacklinkView } from "../../shared/bindings/BacklinkView";
import type { FollowedLinkView } from "../../shared/bindings/FollowedLinkView";
import type { SearchHitView } from "../../shared/bindings/SearchHitView";
import type { SyncReportView } from "../../shared/bindings/SyncReportView";
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
  readNote: (path: string) => call<string>("notes_read_note", { path }),
  search: (query: string) => call<SearchHitView[]>("notes_search", { query }),
  findBacklinks: (path: string) =>
    call<BacklinkView[]>("notes_find_backlinks", { path }),
  saveNote: (path: string, content: string) =>
    call<null>("notes_save_note", { path, content }),
  followLink: (target: string) =>
    call<FollowedLinkView>("notes_follow_link", { target }),
  acknowledgeFlush: (id: number) =>
    call<null>("editor_flush_completed", { id }),
  onFlushRequested: (handler: (id: number) => void): Promise<UnlistenFn> =>
    listen<FlushRequestView>("editor://flush-requested", (event) =>
      handler(event.payload.id),
    ),
  onSynchronized: (
    handler: (changedPaths: string[]) => void,
  ): Promise<UnlistenFn> =>
    listen<SyncReportView>("sync://finished", (event) =>
      handler(event.payload.changedPaths),
    ),
  onChanged: (handler: (paths: string[]) => void): Promise<UnlistenFn> =>
    listen<ChangedPathsView>("notes://changed", (event) =>
      handler(event.payload.paths),
    ),
};
