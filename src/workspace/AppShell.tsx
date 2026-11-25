import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { BaseView } from "../notes/bases/BaseView";
import { isBase } from "../notes/bases/isBase";
import { NoteView } from "../notes/editor/NoteView";
import { notesGateway } from "../notes/gateway/notesGateway";
import { useEditor } from "../notes/store/editorStore";
import { useVault } from "../notes/store/vaultStore";
import { QuickSwitcher } from "../notes/ui/QuickSwitcher";
import { SearchDialog } from "../notes/ui/SearchDialog";
import { TreeDialog, VaultTree } from "../notes/ui/VaultTree";
import {
  reportingErrors,
  useNotifications,
} from "../shared/ui/notificationStore";
import { syncGateway } from "../sync/gateway/syncGateway";
import { useSync } from "../sync/store/syncStore";
import { SyncStatus } from "../sync/ui/SyncStatus";
import styles from "./AppShell.module.css";
import { SettingsDialog } from "./SettingsDialog";
import { TabBar } from "./TabBar";
import { SidePanel } from "./SidePanel";

type Overlay =
  | { kind: "switcher" | "search" | "settings" }
  | { kind: "newNote" | "newFolder"; folder: null };

export function AppShell() {
  const { t } = useTranslation();
  const vault = useVault((state) => state.vault);
  const selected = useVault((state) => state.selected);
  const selectedNote = selected !== null && !isBase(selected) ? selected : null;
  const close = useVault((state) => state.close);
  const [overlay, setOverlay] = useState<Overlay | null>(null);
  const [navigationOpen, setNavigationOpen] = useState(false);

  useEffect(() => {
    void reportingErrors(() => useSync.getState().refresh());
    const subscriptions = [
      notesGateway.onChanged((paths) => {
        void reportingErrors(async () => {
          await useVault.getState().refresh();
          await useEditor.getState().reloadIfChanged(paths);
        });
      }),
      notesGateway.onSynchronized((paths) => {
        void reportingErrors(() => useEditor.getState().reloadIfChanged(paths));
      }),
      notesGateway.onFlushRequested((id) => {
        void useEditor.getState().answerFlushRequest(id);
      }),
      syncGateway.onStarted(() => useSync.getState().started()),
      syncGateway.onFinished((report) => {
        void reportingErrors(() => useSync.getState().finished(report));
      }),
      syncGateway.onFailed((error) => {
        useNotifications.getState().report(error);
        void reportingErrors(() => useSync.getState().failed());
      }),
    ];
    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((stop) => stop());
      }
    };
  }, []);

  useEffect(
    () =>
      useVault.subscribe((state, previous) => {
        if (state.selected !== previous.selected) {
          setNavigationOpen(false);
        }
      }),
    [],
  );

  useEffect(() => {
    if (selectedNote !== null) {
      void reportingErrors(() => useEditor.getState().load(selectedNote));
    }
  }, [selectedNote]);

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key === "Tab") {
        event.preventDefault();
        useVault.getState().cycleTab(event.shiftKey ? -1 : 1);
        return;
      }
      if (!(event.metaKey || event.ctrlKey)) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "w") {
        const active = useVault.getState().selected;
        if (active !== null) {
          event.preventDefault();
          useVault.getState().closeTab(active);
        }
        return;
      }
      if (key === "o") {
        event.preventDefault();
        setOverlay({ kind: "switcher" });
      } else if (key === "f" && event.shiftKey) {
        event.preventDefault();
        setOverlay({ kind: "search" });
      } else if (key === "s") {
        event.preventDefault();
        void reportingErrors(() => useSync.getState().synchronize());
      }
    };
    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, []);

  const closeOverlay = () => setOverlay(null);

  return (
    <div
      className={styles.shell}
      data-testid="app-shell"
      data-with-panel={selectedNote !== null}
      data-navigation={navigationOpen ? "open" : "closed"}
    >
      {navigationOpen && (
        <div
          className={styles.backdrop}
          data-testid="navigation-backdrop"
          onClick={() => setNavigationOpen(false)}
        />
      )}
      <aside className={styles.sidebar}>
        <header className={styles.header}>
          <span className={styles.vaultName}>{vault?.name}</span>
          <button
            type="button"
            title={t("tree.newNote")}
            aria-label={t("tree.newNote")}
            onClick={() => setOverlay({ kind: "newNote", folder: null })}
          >
            ＋
          </button>
          <button
            type="button"
            title={t("tree.newFolder")}
            aria-label={t("tree.newFolder")}
            onClick={() => setOverlay({ kind: "newFolder", folder: null })}
          >
            ⊞
          </button>
          <button
            type="button"
            title={t("settings.title")}
            aria-label={t("settings.title")}
            onClick={() => setOverlay({ kind: "settings" })}
          >
            ⚙
          </button>
          <button
            type="button"
            title={t("layout.closeVault")}
            aria-label={t("layout.closeVault")}
            onClick={close}
          >
            ⏏
          </button>
        </header>
        <VaultTree />
        <SyncStatus />
      </aside>
      <main className={styles.main}>
        <div className={styles.mobileBar}>
          <button
            type="button"
            aria-label={t("layout.showNotes")}
            onClick={() => setNavigationOpen(true)}
          >
            ☰
          </button>
          <span>{vault?.name}</span>
        </div>
        <TabBar />
        {selected !== null && isBase(selected) ? (
          <BaseView key={selected} path={selected} />
        ) : selectedNote ? (
          <NoteView />
        ) : (
          <p className={styles.placeholder}>{t("layout.selectNote")}</p>
        )}
      </main>
      {selectedNote && <SidePanel path={selectedNote} />}
      {overlay?.kind === "switcher" && <QuickSwitcher onClose={closeOverlay} />}
      {overlay?.kind === "search" && <SearchDialog onClose={closeOverlay} />}
      {overlay?.kind === "settings" && (
        <SettingsDialog onClose={closeOverlay} />
      )}
      {(overlay?.kind === "newNote" || overlay?.kind === "newFolder") && (
        <TreeDialog dialog={overlay} onClose={closeOverlay} />
      )}
    </div>
  );
}
