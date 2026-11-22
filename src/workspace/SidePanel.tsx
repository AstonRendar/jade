import { useState } from "react";
import { useTranslation } from "react-i18next";
import { BacklinksPanel } from "../notes/ui/BacklinksPanel";
import { HistoryPanel } from "../sync/ui/HistoryPanel";
import styles from "./SidePanel.module.css";

type Tab = "backlinks" | "history";

export function SidePanel({ path }: { path: string }) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("backlinks");
  const labels: Record<Tab, string> = {
    backlinks: t("navigation.backlinks"),
    history: t("history.title"),
  };
  return (
    <aside className={styles.panel}>
      <div role="tablist" className={styles.tabs}>
        {(["backlinks", "history"] as const).map((option) => (
          <button
            key={option}
            type="button"
            role="tab"
            aria-selected={tab === option}
            onClick={() => setTab(option)}
          >
            {labels[option]}
          </button>
        ))}
      </div>
      <div role="tabpanel" aria-label={labels[tab]} className={styles.content}>
        {tab === "backlinks" ? (
          <BacklinksPanel path={path} />
        ) : (
          <HistoryPanel path={path} />
        )}
      </div>
    </aside>
  );
}
