import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { NotePropertiesView } from "../../shared/bindings/NotePropertiesView";
import type { PropertyView } from "../../shared/bindings/PropertyView";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { notesGateway } from "../gateway/notesGateway";
import { PropertiesPanel } from "../properties/PropertiesPanel";
import { useEditor } from "../store/editorStore";
import { usePropertyCatalog } from "../store/propertyStore";
import { useVault } from "../store/vaultStore";
import { followLink } from "../ui/followLink";
import { MarkdownEditor } from "./MarkdownEditor";
import styles from "./NoteView.module.css";
import { Preview } from "./Preview";

type Mode = "edit" | "preview";
type Head = NotePropertiesView & { content: string };

export function NoteView() {
  const { t } = useTranslation();
  const path = useEditor((state) => state.path);
  const draft = useEditor((state) => state.draft);
  const edit = useEditor((state) => state.edit);
  const entries = useVault((state) => state.entries);
  const catalog = usePropertyCatalog((state) => state.catalog);
  const [mode, setMode] = useState<Mode>("edit");
  const [head, setHead] = useState<Head | null>(null);

  useEffect(() => {
    void reportingErrors(() => usePropertyCatalog.getState().load());
  }, [path]);

  useEffect(() => {
    let current = true;
    void reportingErrors(async () => {
      const read = await notesGateway.readProperties(draft);
      if (current) {
        setHead({ ...read, content: draft });
      }
    });
    return () => {
      current = false;
    };
  }, [draft]);

  if (path === null) {
    return null;
  }
  const notes = entries
    .filter((entry) => entry.kind === "note")
    .map((entry) => entry.path);
  const title = path.slice(path.lastIndexOf("/") + 1).replace(/\.md$/i, "");
  const bodyStart = head?.bodyStart ?? 0;
  const body = draft.slice(bodyStart);

  const editBody = (text: string) => {
    const current = useEditor.getState().draft;
    edit(current.slice(0, bodyStart) + text);
  };
  const changeProperties = (properties: PropertyView[]) => {
    void reportingErrors(async () => {
      const current = useEditor.getState().draft;
      edit(await notesGateway.writeProperties(current, properties));
      await usePropertyCatalog.getState().load();
    });
  };

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
      {head === null ? (
        <div />
      ) : (
        <div className={styles.content}>
          {head.valid ? (
            <PropertiesPanel
              properties={head.properties}
              catalog={catalog}
              editable={mode === "edit"}
              onChange={changeProperties}
              onFollow={followLink}
            />
          ) : (
            <p role="alert" className={styles.invalidProperties}>
              {t("properties.invalid")}
            </p>
          )}
          {mode === "edit" ? (
            <MarkdownEditor
              key={path}
              value={body}
              notes={notes}
              onChange={editBody}
              onFollow={followLink}
            />
          ) : (
            <Preview source={body} onFollow={followLink} />
          )}
        </div>
      )}
    </section>
  );
}
