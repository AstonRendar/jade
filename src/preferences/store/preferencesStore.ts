import { create } from "zustand";
import type { PreferencesView } from "../../shared/bindings/PreferencesView";
import {
  preferencesGateway,
  type PreferencesUpdate,
} from "../gateway/preferencesGateway";

type PreferencesState = {
  preferences: PreferencesView | null;
  load: () => Promise<void>;
  update: (update: PreferencesUpdate) => Promise<void>;
  forgetVault: (path: string) => Promise<void>;
};

export const usePreferences = create<PreferencesState>((set) => ({
  preferences: null,
  load: async () => set({ preferences: await preferencesGateway.get() }),
  update: async (update) =>
    set({ preferences: await preferencesGateway.update(update) }),
  forgetVault: async (path) =>
    set({ preferences: await preferencesGateway.forgetVault(path) }),
}));
