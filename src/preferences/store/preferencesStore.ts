import { create } from "zustand";
import type { PreferencesView } from "../../shared/bindings/PreferencesView";
import { preferencesGateway } from "../gateway/preferencesGateway";

type PreferencesState = {
  preferences: PreferencesView | null;
  load: () => Promise<void>;
  forgetVault: (path: string) => Promise<void>;
};

export const usePreferences = create<PreferencesState>((set) => ({
  preferences: null,
  load: async () => set({ preferences: await preferencesGateway.get() }),
  forgetVault: async (path) =>
    set({ preferences: await preferencesGateway.forgetVault(path) }),
}));
