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
import { useEffect, useRef } from "react";
import styles from "./NoteView.module.css";
import { completeNote, WIKILINK, wikilinkAt } from "./wikilinks";

type MarkdownEditorProps = {
  value: string;
  notes: string[];
  onChange: (text: string) => void;
  onFollow: (target: string) => void;
};

export function MarkdownEditor({
  value,
  notes,
  onChange,
  onFollow,
}: MarkdownEditorProps) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const initialValue = useRef(value);
  const latest = useRef({ notes, onChange, onFollow });

  useEffect(() => {
    latest.current = { notes, onChange, onFollow };
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
          editorTheme,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              latest.current.onChange(update.state.doc.toString());
            }
          }),
          EditorView.domEventHandlers({
            mousedown: (event, editor) =>
              followOnModifierClick(event, editor, latest.current.onFollow),
          }),
        ],
      }),
    });
    return () => view.current?.destroy();
  }, []);

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
});
