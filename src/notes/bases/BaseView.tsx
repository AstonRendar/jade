import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import type { BaseColumnView } from "../../shared/bindings/BaseColumnView";
import type { BaseRowView } from "../../shared/bindings/BaseRowView";
import type { BaseTableView } from "../../shared/bindings/BaseTableView";
import type { CellView } from "../../shared/bindings/CellView";
import type { ErrorView } from "../../shared/bindings/ErrorView";
import { notesGateway } from "../gateway/notesGateway";
import { useVault } from "../store/vaultStore";
import { followLink } from "../ui/followLink";
import styles from "./BaseView.module.css";
import { compareCells } from "./compareCells";
import { BASE_EXTENSION } from "./isBase";

const DEFAULT_COLUMN_WIDTH = 160;
const FILE_LINK_COLUMNS = new Set(["file.name", "file.basename", "file.path"]);

type Sorting = { column: number; descending: boolean } | null;

export function BaseView({ path }: { path: string }) {
  const { t } = useTranslation();
  const [view, setView] = useState<string | null>(null);
  const [table, setTable] = useState<BaseTableView | null>(null);
  const [error, setError] = useState<ErrorView | null>(null);
  const [sorting, setSorting] = useState<Sorting>(null);
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    const stop = notesGateway.onChanged(() =>
      setRevision((value) => value + 1),
    );
    return () => void stop.then((unlisten) => unlisten());
  }, []);

  useEffect(() => {
    let current = true;
    notesGateway.queryBase(path, view).then(
      (result) => {
        if (current) {
          setTable(result);
          setError(null);
        }
      },
      (failure: ErrorView) => {
        if (current) {
          setError(failure);
        }
      },
    );
    return () => {
      current = false;
    };
  }, [path, view, revision]);

  const title = path
    .slice(path.lastIndexOf("/") + 1)
    .replace(BASE_EXTENSION, "");
  const rows = table ? sorted(table.rows, sorting) : [];

  return (
    <section className={styles.base}>
      <header className={styles.header}>
        <h1>{title}</h1>
        {table && table.views.length > 1 && (
          <div role="tablist" className={styles.views}>
            {table.views.map((summary, index) => (
              <button
                key={`${summary.name}-${index}`}
                type="button"
                role="tab"
                aria-selected={table.view === index}
                onClick={() => {
                  setSorting(null);
                  setView(summary.name);
                }}
              >
                {summary.name || t("bases.untitledView")}
              </button>
            ))}
          </div>
        )}
        {table && (
          <span className={styles.count}>
            {t("bases.results", { count: table.total })}
          </span>
        )}
      </header>
      {error ? (
        <p role="alert" className={styles.error}>
          {t(`errors.${error.code}`, error.params)}
        </p>
      ) : (
        table && (
          <div className={styles.scroller}>
            <table
              className={styles.table}
              style={{ width: `${tableWidth(table.columns)}px` }}
            >
              <colgroup>
                {table.columns.map((column) => (
                  <col
                    key={column.key}
                    style={{
                      width: `${column.width ?? DEFAULT_COLUMN_WIDTH}px`,
                    }}
                  />
                ))}
              </colgroup>
              <thead>
                <tr>
                  {table.columns.map((column, index) => (
                    <th
                      key={column.key}
                      scope="col"
                      aria-sort={ariaSort(sorting, index)}
                    >
                      <button
                        type="button"
                        onClick={() => setSorting(nextSorting(sorting, index))}
                      >
                        {columnLabel(column, t)}
                      </button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => (
                  <tr key={row.path}>
                    {row.cells.map((cell, index) => (
                      <td key={table.columns[index]?.key ?? index}>
                        <Cell
                          cell={cell}
                          row={row}
                          opensRow={FILE_LINK_COLUMNS.has(
                            table.columns[index]?.key ?? "",
                          )}
                        />
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )
      )}
    </section>
  );
}

function Cell({
  cell,
  row,
  opensRow,
}: {
  cell: CellView;
  row: BaseRowView;
  opensRow: boolean;
}) {
  const { i18n } = useTranslation();
  switch (cell.type) {
    case "null":
      return null;
    case "boolean":
      return <input type="checkbox" checked={cell.value} readOnly disabled />;
    case "number":
      return <>{new Intl.NumberFormat(i18n.language).format(cell.value)}</>;
    case "date":
      return (
        <>
          {new Intl.DateTimeFormat(i18n.language, {
            dateStyle: "medium",
            timeStyle: "short",
          }).format(new Date(cell.milliseconds))}
        </>
      );
    case "list":
      return (
        <span className={styles.chips}>
          {cell.items.map((item, index) => (
            <span key={index} className={styles.chip}>
              <Cell cell={item} row={row} opensRow={false} />
            </span>
          ))}
        </span>
      );
    case "link":
      return (
        <button
          type="button"
          className={styles.link}
          onClick={() => followLink(cell.target)}
        >
          {cell.label ?? cell.target}
        </button>
      );
    case "file":
      return (
        <button
          type="button"
          className={styles.link}
          onClick={() => open(cell.path)}
        >
          {cell.path.slice(cell.path.lastIndexOf("/") + 1)}
        </button>
      );
    case "text":
      return opensRow ? (
        <button
          type="button"
          className={styles.link}
          onClick={() => open(row.path)}
        >
          {cell.value}
        </button>
      ) : (
        <>{cell.value}</>
      );
  }
}

function open(path: string) {
  useVault.getState().select(path);
}

function columnLabel(column: BaseColumnView, t: TFunction): string {
  if (column.displayName) {
    return column.displayName;
  }
  if (column.key.startsWith("file.")) {
    return t(`bases.properties.${column.key}`, { defaultValue: column.key });
  }
  return column.key.replace(/^(note|formula)\./, "");
}

function tableWidth(columns: BaseColumnView[]): number {
  return columns.reduce(
    (total, column) => total + (column.width ?? DEFAULT_COLUMN_WIDTH),
    0,
  );
}

function nextSorting(current: Sorting, column: number): Sorting {
  if (current?.column !== column) {
    return { column, descending: false };
  }
  return current.descending ? null : { column, descending: true };
}

function ariaSort(sorting: Sorting, column: number) {
  if (sorting?.column !== column) {
    return undefined;
  }
  return sorting.descending ? "descending" : "ascending";
}

function sorted(rows: BaseRowView[], sorting: Sorting): BaseRowView[] {
  if (sorting === null) {
    return rows;
  }
  const { column, descending } = sorting;
  return [...rows].sort((a, b) => {
    const order = compareCells(a.cells[column], b.cells[column]);
    return descending ? -order : order;
  });
}
