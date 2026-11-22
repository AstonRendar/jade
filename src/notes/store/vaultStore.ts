import { create } from "zustand";
import type { EntryView } from "../../shared/bindings/EntryView";
import type { VaultView } from "../../shared/bindings/VaultView";
import { syncGateway } from "../../sync/gateway/syncGateway";
import { notesGateway } from "../gateway/notesGateway";

type VaultState = {
  vault: VaultView | null;
  entries: EntryView[];
  selected: string | null;
  open: (path: string) => Promise<void>;
  clone: (url: string, parent: string) => Promise<void>;
  refresh: () => Promise<void>;
  select: (path: string) => void;
  createNote: (folder: string | null, title: string) => Promise<void>;
  createFolder: (path: string) => Promise<void>;
  rename: (from: string, to: string) => Promise<void>;
  remove: (path: string) => Promise<void>;
  close: () => void;
};

export const useVault = create<VaultState>((set, get) => ({
  vault: null,
  entries: [],
  selected: null,
  open: async (path) => {
    const vault = await notesGateway.openVault(path);
    set({ vault, selected: null, entries: await notesGateway.listEntries() });
  },
  clone: async (url, parent) => {
    const vault = await syncGateway.cloneRepository(url, parent);
    set({ vault, selected: null, entries: await notesGateway.listEntries() });
  },
  refresh: async () => {
    if (get().vault) {
      set({ entries: await notesGateway.listEntries() });
    }
  },
  select: (path) => set({ selected: path }),
  createNote: async (folder, title) => {
    const path = await notesGateway.createNote(folder, title);
    await get().refresh();
    set({ selected: path });
  },
  createFolder: async (path) => {
    await notesGateway.createFolder(path);
    await get().refresh();
  },
  rename: async (from, to) => {
    await notesGateway.renameEntry(from, to);
    const selected = get().selected;
    await get().refresh();
    if (selected === from) {
      set({ selected: to });
    }
  },
  remove: async (path) => {
    await notesGateway.deleteEntry(path);
    const selected = get().selected;
    await get().refresh();
    if (
      selected !== null &&
      (selected === path || selected.startsWith(`${path}/`))
    ) {
      set({ selected: null });
    }
  },
  close: () => set({ vault: null, entries: [], selected: null }),
}));
