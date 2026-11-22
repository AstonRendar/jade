import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { notesGateway } from "../notes/gateway/notesGateway";
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
    const unlisten = notesGateway.onChanged(() => {
      void reportingErrors(() => useVault.getState().refresh());
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

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
        <p className={styles.placeholder}>
          {selected ?? t("layout.selectNote")}
        </p>
      </main>
      {dialog && <TreeDialog dialog={dialog} onClose={() => setDialog(null)} />}
    </div>
  );
}
