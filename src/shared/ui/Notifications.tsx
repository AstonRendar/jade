import { useTranslation } from "react-i18next";
import styles from "./Notifications.module.css";
import { useNotifications } from "./notificationStore";

export function Notifications() {
  const { t } = useTranslation();
  const notifications = useNotifications((state) => state.notifications);
  const dismiss = useNotifications((state) => state.dismiss);
  return (
    <div className={styles.stack} aria-live="polite">
      {notifications.map((notification) => (
        <div
          key={notification.id}
          role={notification.kind === "error" ? "alert" : "status"}
          className={styles.notification}
          data-kind={notification.kind}
        >
          <span>{t(notification.key, notification.params)}</span>
          <button
            type="button"
            aria-label={t("common.dismiss")}
            onClick={() => dismiss(notification.id)}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
