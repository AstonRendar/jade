import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useEditor } from "../store/editorStore";
import { useVault } from "../store/vaultStore";
import { followLink } from "../ui/followLink";
import { MarkdownEditor } from "./MarkdownEditor";
import styles from "./NoteView.module.css";
import { Preview } from "./Preview";

type Mode = "edit" | "preview";

export function NoteView() {
  const { t } = useTranslation();
  const path = useEditor((state) => state.path);
  const draft = useEditor((state) => state.draft);
  const edit = useEditor((state) => state.edit);
  const entries = useVault((state) => state.entries);
  const [mode, setMode] = useState<Mode>("edit");

  if (path === null) {
    return null;
  }
  const notes = entries
    .filter((entry) => entry.kind === "note")
    .map((entry) => entry.path);
  const title = path.slice(path.lastIndexOf("/") + 1).replace(/\.md$/i, "");

  return (
    <section className={styles.note}>
      <header className={styles.header}>
        <h1>{title}</h1>
        <div role="group" className={styles.modes}>
          {(["edit", "preview"] as const).map((option) => (
            <button
              key={option}
              type="button"
              aria-pressed={mode === option}
              onClick={() => setMode(option)}
            >
              {t(`editor.${option}`)}
            </button>
          ))}
        </div>
      </header>
      {mode === "edit" ? (
        <MarkdownEditor
          key={path}
          value={draft}
          notes={notes}
          onChange={edit}
          onFollow={followLink}
        />
      ) : (
        <Preview source={draft} onFollow={followLink} />
      )}
    </section>
  );
}
