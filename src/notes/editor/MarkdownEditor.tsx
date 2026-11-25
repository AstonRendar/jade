import { autocompletion, closeBrackets } from "@codemirror/autocomplete";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { languages } from "@codemirror/language-data";
import { searchKeymap } from "@codemirror/search";
import { EditorState } from "@codemirror/state";
import {
  Decoration,
  drawSelection,
  EditorView,
  keymap,
  MatchDecorator,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { useEffect, useRef, type RefObject } from "react";
import { imageEmbedsIn } from "../images/imageFiles";
import { imagePreview, setImageUrls } from "./imageWidgets";
import styles from "./NoteView.module.css";
import { completeNote, WIKILINK, wikilinkAt } from "./wikilinks";

type MarkdownEditorProps = {
  value: string;
  notes: string[];
  imageUrls: Record<string, string | null>;
  filePicker: RefObject<HTMLInputElement | null>;
  onChange: (text: string) => void;
  onFollow: (target: string) => void;
  onImageTargets: (targets: string[]) => void;
  onAttach: (file: File) => Promise<string | null>;
};

export function MarkdownEditor({
  value,
  notes,
  imageUrls,
  filePicker,
  onChange,
  onFollow,
  onImageTargets,
  onAttach,
}: MarkdownEditorProps) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const initialValue = useRef(value);
  const latest = useRef({
    notes,
    onChange,
    onFollow,
    onImageTargets,
    onAttach,
  });

  useEffect(() => {
    latest.current = { notes, onChange, onFollow, onImageTargets, onAttach };
  });

  useEffect(() => {
    if (!host.current) {
      return;
    }
    view.current = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: initialValue.current,
        extensions: [
          history(),
          drawSelection(),
          closeBrackets(),
          EditorView.lineWrapping,
          markdown({ base: markdownLanguage, codeLanguages: languages }),
          syntaxHighlighting(markdownStyle),
          autocompletion({
            override: [
              (context) => completeNote(context, latest.current.notes),
            ],
          }),
          keymap.of([
            ...defaultKeymap,
            ...historyKeymap,
            ...searchKeymap,
            indentWithTab,
          ]),
          wikilinkHighlighter,
          imagePreview,
          editorTheme,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              const text = update.state.doc.toString();
              latest.current.onChange(text);
              latest.current.onImageTargets(imageTargets(text));
            }
          }),
          EditorView.domEventHandlers({
            mousedown: (event, editor) =>
              followOnModifierClick(event, editor, latest.current.onFollow),
            paste: (event, editor) =>
              attachFiles(
                [...(event.clipboardData?.files ?? [])],
                editor,
                editor.state.selection.main.head,
                latest.current.onAttach,
              ),
            drop: (event, editor) =>
              attachFiles(
                [...(event.dataTransfer?.files ?? [])],
                editor,
                editor.posAtCoords({ x: event.clientX, y: event.clientY }) ??
                  editor.state.selection.main.head,
                latest.current.onAttach,
              ),
          }),
        ],
      }),
    });
    latest.current.onImageTargets(imageTargets(initialValue.current));
    return () => view.current?.destroy();
  }, []);

  useEffect(() => {
    view.current?.dispatch({ effects: setImageUrls.of(imageUrls) });
  }, [imageUrls]);

  useEffect(() => {
    const picker = filePicker.current;
    if (!picker) {
      return;
    }
    const insertPicked = () => {
      const editor = view.current;
      if (editor && picker.files) {
        attachFiles(
          [...picker.files],
          editor,
          editor.state.selection.main.head,
          latest.current.onAttach,
        );
      }
      picker.value = "";
    };
    picker.addEventListener("change", insertPicked);
    return () => picker.removeEventListener("change", insertPicked);
  }, [filePicker]);

  useEffect(() => {
    const editor = view.current;
    if (editor && editor.state.doc.toString() !== value) {
      editor.dispatch({
        changes: { from: 0, to: editor.state.doc.length, insert: value },
      });
    }
  }, [value]);

  return <div ref={host} className={styles.editor} />;
}

function imageTargets(text: string): string[] {
  return text
    .split("\n")
    .flatMap((line) => imageEmbedsIn(line).map((embed) => embed.target));
}

function attachFiles(
  files: File[],
  editor: EditorView,
  position: number,
  onAttach: (file: File) => Promise<string | null>,
): boolean {
  if (files.length === 0) {
    return false;
  }
  void (async () => {
    let at = position;
    for (const file of files) {
      const link = await onAttach(file);
      if (link !== null) {
        const text = `![[${link}]]\n`;
        editor.dispatch({ changes: { from: at, insert: text } });
        at += text.length;
      }
    }
  })();
  return true;
}

function followOnModifierClick(
  event: MouseEvent,
  editor: EditorView,
  onFollow: (target: string) => void,
): boolean {
  if (!(event.metaKey || event.ctrlKey)) {
    return false;
  }
  const position = editor.posAtCoords({ x: event.clientX, y: event.clientY });
  if (position === null) {
    return false;
  }
  const line = editor.state.doc.lineAt(position);
  const target = wikilinkAt(line.text, position - line.from);
  if (target === null) {
    return false;
  }
  event.preventDefault();
  onFollow(target);
  return true;
}

const wikilinkDecorator = new MatchDecorator({
  regexp: WIKILINK,
  decoration: Decoration.mark({ class: "cm-wikilink" }),
});

const wikilinkHighlighter = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(editor: EditorView) {
      this.decorations = wikilinkDecorator.createDeco(editor);
    }
    update(update: ViewUpdate) {
      this.decorations = wikilinkDecorator.updateDeco(update, this.decorations);
    }
  },
  { decorations: (plugin) => plugin.decorations },
);

const markdownStyle = HighlightStyle.define([
  { tag: tags.heading1, fontSize: "1.6em", fontWeight: "700" },
  { tag: tags.heading2, fontSize: "1.35em", fontWeight: "700" },
  { tag: tags.heading3, fontSize: "1.15em", fontWeight: "700" },
  { tag: [tags.heading4, tags.heading5, tags.heading6], fontWeight: "700" },
  { tag: tags.strong, fontWeight: "700" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strikethrough, textDecoration: "line-through" },
  { tag: [tags.link, tags.url], color: "var(--accent-text)" },
  { tag: tags.monospace, fontFamily: "ui-monospace, monospace" },
  { tag: tags.quote, color: "var(--muted)", fontStyle: "italic" },
  { tag: [tags.processingInstruction, tags.meta], color: "var(--muted)" },
]);

const editorTheme = EditorView.theme({
  "&": { height: "100%", color: "var(--text)", backgroundColor: "var(--bg)" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "inherit", lineHeight: "1.6" },
  ".cm-content": {
    maxWidth: "760px",
    margin: "0 auto",
    padding: "24px 16px 40vh",
    caretColor: "var(--text)",
  },
  ".cm-cursor": { borderLeftColor: "var(--text)" },
  ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
    backgroundColor: "var(--selected)",
  },
  ".cm-wikilink": { color: "var(--accent-text)" },
  ".cm-image": { padding: "6px 0" },
  ".cm-image img": {
    maxWidth: "100%",
    height: "auto",
    borderRadius: "var(--radius)",
    display: "block",
  },
});
