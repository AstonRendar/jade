import type { PreferencesView } from "../../shared/bindings/PreferencesView";
import { call } from "../../shared/gateway/call";

export type PreferencesUpdate = {
  locale: string | null;
  theme: string;
  autoSyncMinutes: number;
};

export const preferencesGateway = {
  get: () => call<PreferencesView>("preferences_get"),
  update: (update: PreferencesUpdate) =>
    call<PreferencesView>("preferences_update", update),
  forgetVault: (path: string) =>
    call<PreferencesView>("preferences_forget_vault", { path }),
};
