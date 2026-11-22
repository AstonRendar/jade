import { useEffect, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { usePreferences } from "../preferences/store/preferencesStore";
import dialogStyles from "../shared/ui/Dialog.module.css";
import { reportingErrors } from "../shared/ui/notificationStore";
import { syncGateway } from "../sync/gateway/syncGateway";
import { useSync } from "../sync/store/syncStore";
import styles from "./SettingsDialog.module.css";

const LANGUAGES = [
  ["en", "English"],
  ["es", "Español"],
  ["zh-CN", "简体中文"],
  ["ca", "Català"],
] as const;

export function SettingsDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const preferences = usePreferences((state) => state.preferences);
  const status = useSync((state) => state.status);
  const [locale, setLocale] = useState(preferences?.locale ?? "");
  const [theme, setTheme] = useState(preferences?.theme ?? "system");
  const [minutes, setMinutes] = useState(
    String(preferences?.autoSyncMinutes ?? 5),
  );
  const [token, setToken] = useState("");
  const [hasToken, setHasToken] = useState(false);
  const [remote, setRemote] = useState(status?.remote ?? "");

  useEffect(() => {
    void reportingErrors(async () =>
      setHasToken(await syncGateway.hasAccessToken()),
    );
  }, []);

  const save = (event: FormEvent) => {
    event.preventDefault();
    onClose();
    void reportingErrors(async () => {
      await usePreferences.getState().update({
        locale: locale === "" ? null : locale,
        theme,
        autoSyncMinutes: Number(minutes),
      });
      if (token.trim() !== "") {
        await syncGateway.storeAccessToken(token.trim());
      }
      const trimmedRemote = remote.trim();
      if (
        status?.isRepository &&
        trimmedRemote !== "" &&
        trimmedRemote !== status.remote
      ) {
        await syncGateway.connectRemote(trimmedRemote);
      }
      await useSync.getState().refresh();
    });
  };

  const versionFolder = () =>
    reportingErrors(async () => {
      await syncGateway.initRepository();
      await useSync.getState().refresh();
    });

  const removeToken = () =>
    reportingErrors(async () => {
      await syncGateway.storeAccessToken("");
      setHasToken(false);
    });

  return (
    <div className={dialogStyles.backdrop} onClick={onClose}>
      <form
        role="dialog"
        aria-modal="true"
        aria-label={t("settings.title")}
        className={`${dialogStyles.dialog} ${styles.settings}`}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => event.key === "Escape" && onClose()}
        onSubmit={save}
      >
        <h2>{t("settings.title")}</h2>
        <label>
          {t("settings.language")}
          <select
            value={locale}
            onChange={(event) => setLocale(event.target.value)}
          >
            <option value="">{t("settings.sameAsSystem")}</option>
            {LANGUAGES.map(([code, name]) => (
              <option key={code} value={code}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("settings.theme")}
          <select
            value={theme}
            onChange={(event) => setTheme(event.target.value)}
          >
            <option value="system">{t("settings.sameAsSystem")}</option>
            <option value="light">{t("settings.themeLight")}</option>
            <option value="dark">{t("settings.themeDark")}</option>
          </select>
        </label>
        <label>
          {t("settings.autoSync")}
          <input
            type="number"
            min={0}
            max={1440}
            value={minutes}
            onChange={(event) => setMinutes(event.target.value)}
          />
        </label>
        <label>
          {t("settings.token")}
          <input
            type="password"
            autoComplete="off"
            value={token}
            placeholder={hasToken ? t("settings.tokenStored") : ""}
            onChange={(event) => setToken(event.target.value)}
          />
        </label>
        {hasToken && (
          <button
            type="button"
            className={styles.inline}
            onClick={() => void removeToken()}
          >
            {t("settings.removeToken")}
          </button>
        )}
        {status?.isRepository ? (
          <label>
            {t("settings.remote")}
            <input
              value={remote}
              placeholder="https://github.com/user/notes.git"
              onChange={(event) => setRemote(event.target.value)}
            />
          </label>
        ) : (
          <button
            type="button"
            className={styles.inline}
            onClick={() => void versionFolder()}
          >
            {t("settings.versionFolder")}
          </button>
        )}
        <div className={dialogStyles.actions}>
          <button type="button" onClick={onClose}>
            {t("common.cancel")}
          </button>
          <button type="submit" className={dialogStyles.primary}>
            {t("settings.save")}
          </button>
        </div>
      </form>
    </div>
  );
}
