import { create } from "zustand";
import type { RepositoryStatusView } from "../../shared/bindings/RepositoryStatusView";
import type { SyncReportView } from "../../shared/bindings/SyncReportView";
import { useNotifications } from "../../shared/ui/notificationStore";
import { syncGateway } from "../gateway/syncGateway";

type SyncState = {
  status: RepositoryStatusView | null;
  syncing: boolean;
  refresh: () => Promise<void>;
  synchronize: () => Promise<void>;
  started: () => void;
  finished: (report: SyncReportView) => Promise<void>;
  failed: () => Promise<void>;
};

export const useSync = create<SyncState>((set, get) => ({
  status: null,
  syncing: false,
  refresh: async () => set({ status: await syncGateway.getStatus() }),
  synchronize: async () => {
    set({ syncing: true });
    try {
      await syncGateway.synchronize();
    } finally {
      set({ syncing: false });
    }
  },
  started: () => set({ syncing: true }),
  finished: async (report) => {
    set({ syncing: false });
    for (const conflict of report.conflictCopies) {
      useNotifications.getState().inform("sync.conflictCopy", {
        original: conflict.original,
        copy: conflict.copy,
      });
    }
    await get().refresh();
  },
  failed: async () => {
    set({ syncing: false });
    await get().refresh();
  },
}));
