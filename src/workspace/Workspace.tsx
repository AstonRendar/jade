import type { i18n } from "i18next";
import { useEffect } from "react";
import { I18nextProvider } from "react-i18next";
import { useVault } from "../notes/store/vaultStore";
import { usePreferences } from "../preferences/store/preferencesStore";
import { applyTheme } from "../preferences/ui/applyTheme";
import { resolveLocale } from "../shared/i18n";
import { Notifications } from "../shared/ui/Notifications";
import { reportingErrors } from "../shared/ui/notificationStore";
import { AppShell } from "./AppShell";
import { usePlatform } from "./platformStore";
import { WelcomeScreen } from "./WelcomeScreen";

export function Workspace({ i18n }: { i18n: i18n }) {
  const vault = useVault((state) => state.vault);
  const locale = usePreferences((state) => state.preferences?.locale);
  const theme = usePreferences((state) => state.preferences?.theme);

  useEffect(() => {
    void reportingErrors(() => usePreferences.getState().load());
    void reportingErrors(() => usePlatform.getState().load());
  }, []);

  useEffect(() => {
    void i18n.changeLanguage(locale ?? resolveLocale(navigator.languages));
  }, [i18n, locale]);

  useEffect(() => applyTheme(theme), [theme]);

  return (
    <I18nextProvider i18n={i18n}>
      {vault ? <AppShell /> : <WelcomeScreen />}
      <Notifications />
    </I18nextProvider>
  );
}
