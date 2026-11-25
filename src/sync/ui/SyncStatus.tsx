import { useTranslation } from "react-i18next";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { useSync } from "../store/syncStore";
import styles from "./SyncStatus.module.css";

export function SyncStatus({ onChooseBranch }: { onChooseBranch: () => void }) {
  const { t } = useTranslation();
  const status = useSync((state) => state.status);
  const syncing = useSync((state) => state.syncing);
  return (
    <footer className={styles.status}>
      {status?.isRepository && status.branch !== null && (
        <button
          type="button"
          className={styles.branch}
          title={t("sync.chooseBranch")}
          onClick={onChooseBranch}
        >
          ⎇ {status.branch}
        </button>
      )}
      <span className={styles.summary}>{summary()}</span>
      {status?.isRepository && (
        <button
          type="button"
          disabled={syncing}
          onClick={() =>
            void reportingErrors(() => useSync.getState().synchronize())
          }
        >
          {syncing ? t("sync.syncing") : t("sync.now")}
        </button>
      )}
    </footer>
  );

  function summary(): string {
    if (status === null) {
      return "";
    }
    if (!status.isRepository) {
      return t("sync.notVersioned");
    }
    const parts = [t("sync.pending", { count: status.pendingChanges })];
    if (status.ahead > 0) {
      parts.push(`↑${status.ahead}`);
    }
    if (status.behind > 0) {
      parts.push(`↓${status.behind}`);
    }
    if (status.remote === null) {
      parts.push(t("sync.localOnly"));
    }
    return parts.join(" · ");
  }
}
