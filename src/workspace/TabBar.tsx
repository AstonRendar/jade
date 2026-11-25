import type { MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { useVault } from "../notes/store/vaultStore";
import styles from "./TabBar.module.css";

const MIDDLE_BUTTON = 1;

export function TabBar() {
  const { t } = useTranslation();
  const tabs = useVault((state) => state.tabs);
  const selected = useVault((state) => state.selected);
  const { select, closeTab } = useVault.getState();

  if (tabs.length === 0) {
    return null;
  }

  return (
    <div role="tablist" aria-label={t("tabs.label")} className={styles.bar}>
      {tabs.map((path) => {
        const name = titleOf(path);
        const closeOnMiddleClick = (event: MouseEvent) => {
          if (event.button === MIDDLE_BUTTON) {
            event.preventDefault();
            closeTab(path);
          }
        };
        return (
          <div
            key={path}
            className={styles.tab}
            data-active={path === selected}
            title={path}
            onAuxClick={closeOnMiddleClick}
          >
            <button
              type="button"
              role="tab"
              aria-selected={path === selected}
              className={styles.title}
              onClick={() => select(path)}
            >
              {name}
            </button>
            <button
              type="button"
              className={styles.close}
              aria-label={t("tabs.close", { name })}
              onClick={() => closeTab(path)}
            >
              ×
            </button>
          </div>
        );
      })}
    </div>
  );
}

function titleOf(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1).replace(/\.md$/i, "");
}
