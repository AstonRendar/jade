import { useEffect, useState, type MouseEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { SearchHitView } from "../../shared/bindings/SearchHitView";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { isBase } from "../bases/isBase";
import { notesGateway } from "../gateway/notesGateway";
import { useVault } from "../store/vaultStore";
import { fuzzyMatch } from "./fuzzyMatch";
import styles from "./SearchPanel.module.css";

const SEARCH_DELAY_MS = 200;
const MAX_FILES = 50;

type FileResult = {
  path: string;
  name: string;
  folder: string;
  positions: number[];
};

type HitGroup = { path: string; hits: SearchHitView[] };

export function SearchPanel({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const entries = useVault((state) => state.entries);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHitView[]>([]);

  useEffect(() => {
    if (query.trim() === "") {
      return;
    }
    const timer = setTimeout(() => {
      void reportingErrors(async () =>
        setHits(await notesGateway.search(query)),
      );
    }, SEARCH_DELAY_MS);
    return () => clearTimeout(timer);
  }, [query]);

  const searching = query.trim() !== "";
  const files = searching
    ? matchingFiles(
        entries
          .filter((entry) => entry.kind === "note" || isBase(entry.path))
          .map((entry) => entry.path),
        query,
      )
    : [];
  const groups = searching ? groupByFile(hits) : [];

  return (
    <section className={styles.panel} aria-label={t("search.title")}>
      <input
        className={styles.input}
        type="search"
        autoFocus
        value={query}
        placeholder={t("search.placeholder")}
        aria-label={t("search.placeholder")}
        onChange={(event) => setQuery(event.currentTarget.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            onClose();
          } else if (event.key === "Enter" && files[0]) {
            open(files[0].path, event.metaKey || event.ctrlKey);
          }
        }}
      />
      <div className={styles.results}>
        {searching && (
          <>
            <h3 className={styles.heading}>
              {t("search.files")}
              <span className={styles.count}>{files.length}</span>
            </h3>
            <ul className={styles.list}>
              {files.map((file) => (
                <li key={file.path}>
                  <button
                    type="button"
                    className={styles.file}
                    title={file.path}
                    onClick={(event) => openFrom(event, file.path)}
                  >
                    <span className={styles.name}>
                      {highlightPositions(file.name, file.positions)}
                    </span>
                    {file.folder && (
                      <span className={styles.folder}>{file.folder}</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
            <h3 className={styles.heading}>
              {t("search.content")}
              <span className={styles.count}>
                {groups.reduce((total, group) => total + group.hits.length, 0)}
              </span>
            </h3>
            {groups.map((group) => (
              <div key={group.path} className={styles.group}>
                <button
                  type="button"
                  className={styles.groupTitle}
                  title={group.path}
                  onClick={(event) => openFrom(event, group.path)}
                >
                  {titleOf(group.path)}
                  <span className={styles.count}>{group.hits.length}</span>
                </button>
                <ul className={styles.list}>
                  {group.hits.map((hit) => (
                    <li key={hit.line}>
                      <button
                        type="button"
                        className={styles.hit}
                        onClick={(event) => openFrom(event, group.path)}
                      >
                        {highlightText(hit.text, query)}
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </>
        )}
        {!searching && <p className={styles.hint}>{t("search.hint")}</p>}
      </div>
    </section>
  );
}

function openFrom(event: MouseEvent, path: string) {
  open(path, event.metaKey || event.ctrlKey);
}

function open(path: string, inNewTab: boolean) {
  if (inNewTab) {
    useVault.getState().openInNewTab(path);
  } else {
    useVault.getState().select(path);
  }
}

export function matchingFiles(paths: string[], query: string): FileResult[] {
  return paths
    .flatMap((path) => {
      const name = titleOf(path);
      const byName = fuzzyMatch(query, name);
      const byPath =
        byName || !query.includes("/") ? null : fuzzyMatch(query, path);
      if (!byName && !byPath) {
        return [];
      }
      const score = byName ? byName.score + 100 : (byPath?.score ?? 0);
      const folder = path.includes("/")
        ? path.slice(0, path.lastIndexOf("/"))
        : "";
      return [
        { path, name, folder, positions: byName?.positions ?? [], score },
      ];
    })
    .sort((a, b) => b.score - a.score || a.name.localeCompare(b.name))
    .slice(0, MAX_FILES)
    .map(({ path, name, folder, positions }) => ({
      path,
      name,
      folder,
      positions,
    }));
}

function groupByFile(hits: SearchHitView[]): HitGroup[] {
  const groups: HitGroup[] = [];
  for (const hit of hits) {
    if (hit.line === 0) {
      continue;
    }
    const group = groups.find((candidate) => candidate.path === hit.path);
    if (group) {
      group.hits.push(hit);
    } else {
      groups.push({ path: hit.path, hits: [hit] });
    }
  }
  return groups;
}

function highlightPositions(text: string, positions: number[]): ReactNode[] {
  const marked = new Set(positions);
  return [...text].map((character, index) =>
    marked.has(index) ? <mark key={index}>{character}</mark> : character,
  );
}

function highlightText(text: string, query: string): ReactNode[] {
  const needle = query.trim().toLocaleLowerCase();
  const lower = text.toLocaleLowerCase();
  const parts: ReactNode[] = [];
  let from = 0;
  for (
    let at = lower.indexOf(needle);
    needle && at !== -1;
    at = lower.indexOf(needle, from)
  ) {
    parts.push(
      text.slice(from, at),
      <mark key={at}>{text.slice(at, at + needle.length)}</mark>,
    );
    from = at + needle.length;
  }
  parts.push(text.slice(from));
  return parts;
}

function titleOf(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1).replace(/\.md$/i, "");
}
