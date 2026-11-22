import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useEditor } from "../../notes/store/editorStore";
import styles from "../../notes/ui/SidePanel.module.css";
import type { RevisionView } from "../../shared/bindings/RevisionView";
import dialogStyles from "../../shared/ui/Dialog.module.css";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { syncGateway } from "../gateway/syncGateway";

type OpenRevision = { revision: RevisionView; content: string };

export function HistoryPanel({ path }: { path: string }) {
  const { t, i18n } = useTranslation();
  const [revisions, setRevisions] = useState<RevisionView[]>([]);
  const [open, setOpen] = useState<OpenRevision | null>(null);
  const format = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });

  useEffect(() => {
    void reportingErrors(async () =>
      setRevisions(await syncGateway.listNoteHistory(path)),
    );
  }, [path]);

  const show = (revision: RevisionView) =>
    reportingErrors(async () =>
      setOpen({
        revision,
        content: await syncGateway.readNoteRevision(path, revision.id),
      }),
    );

  const restore = (content: string) => {
    setOpen(null);
    useEditor.getState().edit(content);
    void reportingErrors(() => useEditor.getState().flush());
  };

  return (
    <>
      {revisions.length === 0 ? (
        <p className={styles.empty}>{t("history.empty")}</p>
      ) : (
        <ul className={styles.list}>
          {revisions.map((revision) => (
            <li key={revision.id}>
              <button type="button" onClick={() => void show(revision)}>
                <strong>{revision.summary}</strong>
                <span>
                  {format.format(new Date(revision.epochSeconds * 1000))} ·{" "}
                  {revision.author}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {open && (
        <div className={dialogStyles.backdrop} onClick={() => setOpen(null)}>
          <div
            role="dialog"
            aria-modal="true"
            aria-label={open.revision.summary}
            className={dialogStyles.dialog}
            onClick={(event) => event.stopPropagation()}
          >
            <h2>{open.revision.summary}</h2>
            <pre className={dialogStyles.revision}>{open.content}</pre>
            <div className={dialogStyles.actions}>
              <button type="button" onClick={() => setOpen(null)}>
                {t("common.close")}
              </button>
              <button
                type="button"
                className={dialogStyles.primary}
                onClick={() => restore(open.content)}
              >
                {t("history.restore")}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
