import { open } from "@tauri-apps/plugin-dialog";
import { useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { useVault } from "../notes/store/vaultStore";
import { usePreferences } from "../preferences/store/preferencesStore";
import type { CapabilitiesView } from "../shared/bindings/CapabilitiesView";
import { isErrorView } from "../shared/gateway/call";
import { BusyOverlay } from "../shared/ui/BusyOverlay";
import { PromptDialog } from "../shared/ui/Dialog";
import {
  reportingErrors,
  useNotifications,
} from "../shared/ui/notificationStore";
import { usePlatform } from "./platformStore";
import { SettingsDialog } from "./SettingsDialog";
import styles from "./WelcomeScreen.module.css";

export function WelcomeScreen() {
  const { t } = useTranslation();
  const preferences = usePreferences((state) => state.preferences);
  const capabilities = usePlatform((state) => state.capabilities);
  const recentVaults = preferences?.recentVaults ?? [];
  const [form, setForm] = useState<"clone" | "create" | "settings" | null>(
    null,
  );
  const [cloning, setCloning] = useState(false);

  const openVault = async (path: string) => {
    try {
      await useVault.getState().open(path);
    } catch (error) {
      useNotifications.getState().report(error);
      if (isErrorView(error) && error.code === "vault_not_found") {
        await reportingErrors(() =>
          usePreferences.getState().forgetVault(path),
        );
      }
    }
  };

  const chooseFolder = async () => {
    const folder = await open({
      directory: true,
      title: t("welcome.openFolder"),
    });
    if (typeof folder === "string") {
      await openVault(folder);
    }
  };

  const clone = async (capabilities: CapabilitiesView, url: string) => {
    const parent = await destinationFolder(
      capabilities,
      t("welcome.cloneDestination"),
    );
    if (parent === null) {
      return;
    }
    setCloning(true);
    await reportingErrors(() => useVault.getState().clone(url, parent));
    setCloning(false);
  };

  return (
    <>
      <main className={styles.welcome} inert={cloning}>
        <button
          type="button"
          className={styles.settings}
          title={t("settings.title")}
          aria-label={t("settings.title")}
          onClick={() => setForm("settings")}
        >
          ⚙
        </button>
        {form === "settings" && (
          <SettingsDialog onClose={() => setForm(null)} />
        )}
        <h1>{t("welcome.title")}</h1>
        <p className={styles.subtitle}>{t("welcome.subtitle")}</p>
        {capabilities && (
          <div className={styles.actions}>
            {capabilities.canOpenFolders && (
              <button
                type="button"
                className={styles.primary}
                onClick={chooseFolder}
              >
                {t("welcome.openFolder")}
              </button>
            )}
            <button type="button" onClick={() => setForm("create")}>
              {t("welcome.createVault")}
            </button>
            <button type="button" onClick={() => setForm("clone")}>
              {t("welcome.cloneRepository")}
            </button>
          </div>
        )}
        {capabilities && form === "clone" && (
          <CloneForm
            capabilities={capabilities}
            onClone={(url) => clone(capabilities, url)}
            onCancel={() => setForm(null)}
          />
        )}
        {capabilities && form === "create" && (
          <PromptDialog
            title={t("welcome.vaultName")}
            submitLabel={t("common.create")}
            onSubmit={(name) => {
              setForm(null);
              void createVault(
                capabilities,
                name,
                t("welcome.createVaultDestination"),
              );
            }}
            onCancel={() => setForm(null)}
          />
        )}
        <section className={styles.recent}>
          <h2>{t("welcome.recentVaults")}</h2>
          {recentVaults.length === 0 ? (
            <p className={styles.muted}>{t("welcome.noRecentVaults")}</p>
          ) : (
            <ul>
              {recentVaults.map((path) => (
                <li key={path}>
                  <button type="button" onClick={() => openVault(path)}>
                    <strong>{lastSegment(path)}</strong>
                    <span className={styles.muted}>{path}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      </main>
      {cloning && (
        <BusyOverlay
          message={t("welcome.cloning")}
          detail={t("welcome.cloningDetail")}
        />
      )}
    </>
  );
}

async function createVault(
  capabilities: CapabilitiesView,
  name: string,
  title: string,
) {
  const parent = await destinationFolder(capabilities, title);
  if (parent !== null) {
    await reportingErrors(() => useVault.getState().createVault(parent, name));
  }
}

async function destinationFolder(
  capabilities: CapabilitiesView,
  title: string,
): Promise<string | null> {
  if (capabilities.managedVaultsFolder !== null) {
    return capabilities.managedVaultsFolder;
  }
  const folder = await open({ directory: true, title });
  return typeof folder === "string" ? folder : null;
}

function CloneForm({
  capabilities,
  onClone,
  onCancel,
}: {
  capabilities: CapabilitiesView;
  onClone: (url: string) => Promise<void>;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const [url, setUrl] = useState("");
  const submit = (event: FormEvent) => {
    event.preventDefault();
    void onClone(url.trim());
  };
  return (
    <form className={styles.clone} onSubmit={submit}>
      <label>
        {t("welcome.cloneUrl")}
        <input
          autoFocus
          required
          value={url}
          placeholder="https://github.com/user/notes.git"
          onChange={(event) => setUrl(event.target.value)}
        />
      </label>
      <div className={styles.actions}>
        <button type="button" onClick={onCancel}>
          {t("common.cancel")}
        </button>
        <button type="submit" className={styles.primary}>
          {capabilities.managedVaultsFolder === null
            ? t("welcome.chooseFolderAndClone")
            : t("welcome.clone")}
        </button>
      </div>
    </form>
  );
}

function lastSegment(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}
