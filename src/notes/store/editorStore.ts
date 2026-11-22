import { create } from "zustand";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { notesGateway } from "../gateway/notesGateway";

export const AUTOSAVE_DELAY_MS = 600;

type EditorState = {
  path: string | null;
  saved: string;
  draft: string;
  isDirty: () => boolean;
  load: (path: string) => Promise<void>;
  edit: (text: string) => void;
  flush: () => Promise<void>;
  reloadIfChanged: (paths: string[]) => Promise<void>;
  answerFlushRequest: (id: number) => Promise<void>;
  reset: () => void;
};

let autosave: ReturnType<typeof setTimeout> | undefined;

function cancelAutosave() {
  clearTimeout(autosave);
  autosave = undefined;
}

export const useEditor = create<EditorState>((set, get) => ({
  path: null,
  saved: "",
  draft: "",
  isDirty: () => get().draft !== get().saved,
  load: async (path) => {
    await get().flush();
    const content = await notesGateway.readNote(path);
    set({ path, saved: content, draft: content });
  },
  edit: (text) => {
    set({ draft: text });
    cancelAutosave();
    autosave = setTimeout(() => {
      void reportingErrors(() => get().flush());
    }, AUTOSAVE_DELAY_MS);
  },
  flush: async () => {
    cancelAutosave();
    const { path, draft } = get();
    if (path === null || !get().isDirty()) {
      return;
    }
    await notesGateway.saveNote(path, draft);
    if (get().path === path) {
      set({ saved: draft });
    }
  },
  reloadIfChanged: async (paths) => {
    const { path } = get();
    if (path === null || !paths.includes(path) || get().isDirty()) {
      return;
    }
    const content = await notesGateway.readNote(path);
    if (get().path === path && !get().isDirty()) {
      set({ saved: content, draft: content });
    }
  },
  answerFlushRequest: async (id) => {
    await reportingErrors(() => get().flush());
    await notesGateway.acknowledgeFlush(id);
  },
  reset: () => {
    cancelAutosave();
    set({ path: null, saved: "", draft: "" });
  },
}));
