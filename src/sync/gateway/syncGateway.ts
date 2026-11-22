import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ErrorView } from "../../shared/bindings/ErrorView";
import type { RepositoryStatusView } from "../../shared/bindings/RepositoryStatusView";
import type { RevisionView } from "../../shared/bindings/RevisionView";
import type { SyncReportView } from "../../shared/bindings/SyncReportView";
import type { VaultView } from "../../shared/bindings/VaultView";
import { call } from "../../shared/gateway/call";

export const syncGateway = {
  cloneRepository: (url: string, parent: string) =>
    call<VaultView>("sync_clone_repository", { url, parent }),
  synchronize: () => call<SyncReportView>("sync_synchronize"),
  getStatus: () => call<RepositoryStatusView>("sync_get_status"),
  initRepository: () => call<null>("sync_init_repository"),
  connectRemote: (url: string) => call<null>("sync_connect_remote", { url }),
  listNoteHistory: (path: string) =>
    call<RevisionView[]>("sync_list_note_history", { path }),
  readNoteRevision: (path: string, commit: string) =>
    call<string>("sync_read_note_revision", { path, commit }),
  storeAccessToken: (token: string) =>
    call<null>("sync_store_access_token", { token }),
  hasAccessToken: () => call<boolean>("sync_has_access_token"),
  onStarted: (handler: () => void): Promise<UnlistenFn> =>
    listen("sync://started", () => handler()),
  onFinished: (
    handler: (report: SyncReportView) => void,
  ): Promise<UnlistenFn> =>
    listen<SyncReportView>("sync://finished", (event) =>
      handler(event.payload),
    ),
  onFailed: (handler: (error: ErrorView) => void): Promise<UnlistenFn> =>
    listen<ErrorView>("sync://failed", (event) => handler(event.payload)),
};
