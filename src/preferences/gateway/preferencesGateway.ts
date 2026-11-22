import type { PreferencesView } from "../../shared/bindings/PreferencesView";
import { call } from "../../shared/gateway/call";

export const preferencesGateway = {
  get: () => call<PreferencesView>("preferences_get"),
  forgetVault: (path: string) =>
    call<PreferencesView>("preferences_forget_vault", { path }),
};
