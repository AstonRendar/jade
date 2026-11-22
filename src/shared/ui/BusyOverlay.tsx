import styles from "./BusyOverlay.module.css";

export function BusyOverlay({
  message,
  detail,
}: {
  message: string;
  detail: string;
}) {
  return (
    <div className={styles.overlay}>
      <div className={styles.panel} role="status" aria-live="polite">
        <span className={styles.spinner} aria-hidden="true" />
        <strong>{message}</strong>
        <span className={styles.detail}>{detail}</span>
      </div>
    </div>
  );
}
