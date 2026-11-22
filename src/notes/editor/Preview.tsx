import { openUrl } from "@tauri-apps/plugin-opener";
import { useMemo, type MouseEvent } from "react";
import styles from "./NoteView.module.css";
import { renderMarkdown } from "./renderMarkdown";

type PreviewProps = {
  source: string;
  onFollow: (target: string) => void;
};

export function Preview({ source, onFollow }: PreviewProps) {
  const html = useMemo(() => renderMarkdown(source), [source]);
  const handleClick = (event: MouseEvent<HTMLDivElement>) => {
    const anchor = (event.target as HTMLElement).closest("a");
    if (!anchor) {
      return;
    }
    event.preventDefault();
    const target = anchor.dataset.target;
    if (target) {
      onFollow(target);
    } else if (/^https?:\/\//.test(anchor.href)) {
      void openUrl(anchor.href);
    }
  };
  return (
    <article
      className={styles.preview}
      onClick={handleClick}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
