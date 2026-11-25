import { create } from "zustand";
import type { EntryView } from "../../shared/bindings/EntryView";
import type { VaultView } from "../../shared/bindings/VaultView";
import { syncGateway } from "../../sync/gateway/syncGateway";
import { notesGateway } from "../gateway/notesGateway";
import { useEditor } from "./editorStore";

type VaultState = {
  vault: VaultView | null;
  entries: EntryView[];
  selected: string | null;
  tabs: string[];
  open: (path: string) => Promise<void>;
  createVault: (parent: string, name: string) => Promise<void>;
  clone: (url: string, parent: string) => Promise<void>;
  refresh: () => Promise<void>;
  select: (path: string) => void;
  openInNewTab: (path: string) => void;
  closeTab: (path: string) => void;
  cycleTab: (step: number) => void;
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
  tabs: [],
  open: async (path) => {
    const vault = await notesGateway.openVault(path);
    set({
      vault,
      selected: null,
      tabs: [],
      entries: await notesGateway.listEntries(),
    });
  },
  createVault: async (parent, name) => {
    const vault = await notesGateway.createVault(parent, name);
    set({
      vault,
      selected: null,
      tabs: [],
      entries: await notesGateway.listEntries(),
    });
  },
  clone: async (url, parent) => {
    const vault = await syncGateway.cloneRepository(url, parent);
    set({
      vault,
      selected: null,
      tabs: [],
      entries: await notesGateway.listEntries(),
    });
  },
  refresh: async () => {
    if (get().vault) {
      set({ entries: await notesGateway.listEntries() });
    }
  },
  select: (path) =>
    set(({ tabs, selected }) => ({
      selected: path,
      tabs: tabs.includes(path) ? tabs : replaceTab(tabs, selected, path),
    })),
  openInNewTab: (path) =>
    set(({ tabs, selected }) => ({
      selected: path,
      tabs: tabs.includes(path) ? tabs : insertAfter(tabs, selected, path),
    })),
  closeTab: (path) =>
    set(({ tabs, selected }) => {
      const index = tabs.indexOf(path);
      const remaining = tabs.filter((tab) => tab !== path);
      if (selected !== path) {
        return { tabs: remaining };
      }
      return {
        tabs: remaining,
        selected: remaining[Math.min(index, remaining.length - 1)] ?? null,
      };
    }),
  cycleTab: (step) => {
    const { tabs, selected } = get();
    if (tabs.length === 0) {
      return;
    }
    const index = selected === null ? 0 : tabs.indexOf(selected);
    const next = (index + step + tabs.length) % tabs.length;
    set({ selected: tabs[next] ?? null });
  },
  createNote: async (folder, title) => {
    const path = await notesGateway.createNote(folder, title);
    await get().refresh();
    get().select(path);
  },
  createFolder: async (path) => {
    await notesGateway.createFolder(path);
    await get().refresh();
  },
  rename: async (from, to) => {
    await useEditor.getState().flush();
    await notesGateway.renameEntry(from, to);
    await get().refresh();
    set(({ tabs, selected }) => ({
      tabs: tabs.map((tab) => moved(tab, from, to)),
      selected: selected === null ? null : moved(selected, from, to),
    }));
  },
  remove: async (path) => {
    await notesGateway.deleteEntry(path);
    await get().refresh();
    const inside = (tab: string) => tab === path || tab.startsWith(`${path}/`);
    const { tabs, selected } = get();
    const remaining = tabs.filter((tab) => !inside(tab));
    set({
      tabs: remaining,
      selected:
        selected !== null && inside(selected)
          ? (remaining[0] ?? null)
          : selected,
    });
  },
  close: () => set({ vault: null, entries: [], selected: null, tabs: [] }),
}));

function replaceTab(
  tabs: string[],
  current: string | null,
  path: string,
): string[] {
  const index = current === null ? -1 : tabs.indexOf(current);
  if (index === -1) {
    return [...tabs, path];
  }
  return tabs.map((tab, at) => (at === index ? path : tab));
}

function insertAfter(
  tabs: string[],
  current: string | null,
  path: string,
): string[] {
  const index = current === null ? tabs.length - 1 : tabs.indexOf(current);
  return [...tabs.slice(0, index + 1), path, ...tabs.slice(index + 1)];
}

function moved(path: string, from: string, to: string): string {
  if (path === from) {
    return to;
  }
  return path.startsWith(`${from}/`) ? `${to}${path.slice(from.length)}` : path;
}
