import { useMemo, useState, type MouseEvent } from "react";
import { useTranslation } from "react-i18next";
import { ConfirmDialog, PromptDialog } from "../../shared/ui/Dialog";
import { ContextMenu, type MenuItem } from "../../shared/ui/ContextMenu";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { isBase } from "../bases/isBase";
import { useVault } from "../store/vaultStore";
import { buildTree, parentOf, type TreeNode } from "./buildTree";
import styles from "./VaultTree.module.css";

type Dialog =
  | { kind: "newNote"; folder: string | null }
  | { kind: "newFolder"; folder: string | null }
  | { kind: "rename"; node: TreeNode }
  | { kind: "delete"; node: TreeNode };

type Menu = { x: number; y: number; node: TreeNode };

export function VaultTree() {
  const { t } = useTranslation();
  const entries = useVault((state) => state.entries);
  const tree = useMemo(() => buildTree(entries), [entries]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [menu, setMenu] = useState<Menu | null>(null);
  const [dialog, setDialog] = useState<Dialog | null>(null);

  const toggle = (path: string) =>
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });

  const openMenu = (event: MouseEvent, node: TreeNode) => {
    event.preventDefault();
    setMenu({ x: event.clientX, y: event.clientY, node });
  };

  return (
    <nav className={styles.tree}>
      {tree.length === 0 ? (
        <p className={styles.empty}>{t("tree.empty")}</p>
      ) : (
        <ul role="tree" aria-label={t("tree.label")}>
          {tree.map((node) => (
            <TreeItem
              key={node.path}
              node={node}
              depth={0}
              expanded={expanded}
              onToggle={toggle}
              onMenu={openMenu}
            />
          ))}
        </ul>
      )}
      {menu && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          items={menuItems(menu.node)}
          onClose={() => setMenu(null)}
        />
      )}
      {dialog && <TreeDialog dialog={dialog} onClose={() => setDialog(null)} />}
    </nav>
  );

  function menuItems(node: TreeNode): MenuItem[] {
    const folder =
      node.kind === "folder" ? node.path : parentOf(node.path) || null;
    return [
      {
        label: t("tree.newNote"),
        action: () => setDialog({ kind: "newNote", folder }),
      },
      {
        label: t("tree.newFolder"),
        action: () => setDialog({ kind: "newFolder", folder }),
      },
      {
        label: t("tree.rename"),
        action: () => setDialog({ kind: "rename", node }),
      },
      {
        label: t("tree.delete"),
        action: () => setDialog({ kind: "delete", node }),
      },
    ];
  }
}

type TreeItemProps = {
  node: TreeNode;
  depth: number;
  expanded: Set<string>;
  onToggle: (path: string) => void;
  onMenu: (event: MouseEvent, node: TreeNode) => void;
};

function TreeItem({ node, depth, expanded, onToggle, onMenu }: TreeItemProps) {
  const selected = useVault((state) => state.selected);
  const select = useVault((state) => state.select);
  const isFolder = node.kind === "folder";
  const isOpen = expanded.has(node.path);
  const activate = () => {
    if (isFolder) {
      onToggle(node.path);
    } else if (opens(node)) {
      select(node.path);
    }
  };
  return (
    <li
      role="treeitem"
      aria-label={node.name}
      aria-expanded={isFolder ? isOpen : undefined}
      aria-selected={opens(node) ? selected === node.path : undefined}
      tabIndex={0}
      onClick={(event) => {
        event.stopPropagation();
        activate();
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter") {
          event.stopPropagation();
          activate();
        }
      }}
      onContextMenu={(event) => {
        event.stopPropagation();
        onMenu(event, node);
      }}
    >
      <span
        className={styles.row}
        data-kind={node.kind}
        data-selected={selected === node.path}
        style={{ paddingInlineStart: `${8 + depth * 14}px` }}
      >
        {isFolder && (
          <span className={styles.chevron}>{isOpen ? "▾" : "▸"}</span>
        )}
        {node.name}
      </span>
      {isFolder && isOpen && (
        <ul role="group">
          {node.children.map((child) => (
            <TreeItem
              key={child.path}
              node={child}
              depth={depth + 1}
              expanded={expanded}
              onToggle={onToggle}
              onMenu={onMenu}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

function opens(node: TreeNode): boolean {
  return node.kind === "note" || isBase(node.path);
}

export function TreeDialog({
  dialog,
  onClose,
}: {
  dialog: Dialog;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const { createNote, createFolder, rename, remove } = useVault.getState();
  const run = (action: () => Promise<void>) => {
    onClose();
    void reportingErrors(action);
  };
  switch (dialog.kind) {
    case "newNote":
      return (
        <PromptDialog
          title={t("tree.newNote")}
          submitLabel={t("common.create")}
          onSubmit={(title) => run(() => createNote(dialog.folder, title))}
          onCancel={onClose}
        />
      );
    case "newFolder":
      return (
        <PromptDialog
          title={t("tree.newFolder")}
          submitLabel={t("common.create")}
          onSubmit={(name) =>
            run(() => createFolder(join(dialog.folder, name)))
          }
          onCancel={onClose}
        />
      );
    case "rename":
      return (
        <PromptDialog
          title={t("tree.rename")}
          initialValue={dialog.node.name}
          submitLabel={t("tree.rename")}
          onSubmit={(name) =>
            run(() => rename(dialog.node.path, renamed(dialog.node, name)))
          }
          onCancel={onClose}
        />
      );
    case "delete":
      return (
        <ConfirmDialog
          title={t("tree.delete")}
          message={t("tree.deleteConfirm", { name: dialog.node.name })}
          confirmLabel={t("tree.delete")}
          onConfirm={() => run(() => remove(dialog.node.path))}
          onCancel={onClose}
        />
      );
  }
}

function join(folder: string | null, name: string): string {
  return folder ? `${folder}/${name}` : name;
}

function renamed(node: TreeNode, name: string): string {
  const fileName = node.kind === "note" ? `${name}.md` : name;
  return join(parentOf(node.path) || null, fileName);
}
