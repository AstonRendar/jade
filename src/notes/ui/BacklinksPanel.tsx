import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { BacklinkView } from "../../shared/bindings/BacklinkView";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { notesGateway } from "../gateway/notesGateway";
import { useVault } from "../store/vaultStore";
import styles from "./SidePanel.module.css";

export function BacklinksPanel({ path }: { path: string }) {
  const { t } = useTranslation();
  const entries = useVault((state) => state.entries);
  const [backlinks, setBacklinks] = useState<BacklinkView[]>([]);

  useEffect(() => {
    void reportingErrors(async () =>
      setBacklinks(await notesGateway.findBacklinks(path)),
    );
  }, [path, entries]);

  if (backlinks.length === 0) {
    return <p className={styles.empty}>{t("navigation.noBacklinks")}</p>;
  }
  return (
    <ul className={styles.list}>
      {backlinks.map((backlink) => (
        <li key={`${backlink.source}:${backlink.line}`}>
          <button
            type="button"
            onClick={() => useVault.getState().select(backlink.source)}
          >
            <strong>{backlink.source}</strong>
            <span>{backlink.text}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
