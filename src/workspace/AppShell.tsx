import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { NoteView } from "../notes/editor/NoteView";
import { notesGateway } from "../notes/gateway/notesGateway";
import { useEditor } from "../notes/store/editorStore";
import { useVault } from "../notes/store/vaultStore";
import { TreeDialog, VaultTree } from "../notes/ui/VaultTree";
import { reportingErrors } from "../shared/ui/notificationStore";
import styles from "./AppShell.module.css";

type RootDialog = { kind: "newNote" | "newFolder"; folder: null };

export function AppShell() {
  const { t } = useTranslation();
  const vault = useVault((state) => state.vault);
  const selected = useVault((state) => state.selected);
  const close = useVault((state) => state.close);
  const [dialog, setDialog] = useState<RootDialog | null>(null);

  useEffect(() => {
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
    ];
    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((stop) => stop());
      }
    };
  }, []);

  useEffect(() => {
    if (selected !== null) {
      void reportingErrors(() => useEditor.getState().load(selected));
    }
  }, [selected]);

  return (
    <div className={styles.shell}>
      <aside className={styles.sidebar}>
        <header className={styles.header}>
          <span className={styles.vaultName}>{vault?.name}</span>
          <button
            type="button"
            title={t("tree.newNote")}
            aria-label={t("tree.newNote")}
            onClick={() => setDialog({ kind: "newNote", folder: null })}
          >
            ＋
          </button>
          <button
            type="button"
            title={t("tree.newFolder")}
            aria-label={t("tree.newFolder")}
            onClick={() => setDialog({ kind: "newFolder", folder: null })}
          >
            ⊞
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
      </aside>
      <main className={styles.main}>
        {selected ? (
          <NoteView />
        ) : (
          <p className={styles.placeholder}>{t("layout.selectNote")}</p>
        )}
      </main>
      {dialog && <TreeDialog dialog={dialog} onClose={() => setDialog(null)} />}
    </div>
  );
}
