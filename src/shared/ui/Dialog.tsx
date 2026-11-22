import { useState, type FormEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import styles from "./Dialog.module.css";

type PromptDialogProps = {
  title: string;
  initialValue?: string;
  submitLabel: string;
  onSubmit: (value: string) => void;
  onCancel: () => void;
};

export function PromptDialog({
  title,
  initialValue = "",
  submitLabel,
  onSubmit,
  onCancel,
}: PromptDialogProps) {
  const { t } = useTranslation();
  const [value, setValue] = useState(initialValue);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (value.trim() !== "") {
      onSubmit(value.trim());
    }
  };
  return (
    <DialogFrame title={title} onCancel={onCancel}>
      <form onSubmit={submit} className={styles.body}>
        <input
          autoFocus
          aria-label={title}
          value={value}
          onChange={(event) => setValue(event.target.value)}
        />
        <div className={styles.actions}>
          <button type="button" onClick={onCancel}>
            {t("common.cancel")}
          </button>
          <button type="submit" className={styles.primary}>
            {submitLabel}
          </button>
        </div>
      </form>
    </DialogFrame>
  );
}

type ConfirmDialogProps = {
  title: string;
  message: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
};

export function ConfirmDialog({
  title,
  message,
  confirmLabel,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const { t } = useTranslation();
  return (
    <DialogFrame title={title} onCancel={onCancel}>
      <div className={styles.body}>
        <p>{message}</p>
        <div className={styles.actions}>
          <button type="button" onClick={onCancel}>
            {t("common.cancel")}
          </button>
          <button type="button" className={styles.danger} onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </DialogFrame>
  );
}

function DialogFrame({
  title,
  onCancel,
  children,
}: {
  title: string;
  onCancel: () => void;
  children: ReactNode;
}) {
  return (
    <div
      className={styles.backdrop}
      onKeyDown={(event) => event.key === "Escape" && onCancel()}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className={styles.dialog}
      >
        <h2>{title}</h2>
        {children}
      </div>
    </div>
  );
}
