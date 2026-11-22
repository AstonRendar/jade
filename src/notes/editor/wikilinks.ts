import type {
  Completion,
  CompletionContext,
  CompletionResult,
} from "@codemirror/autocomplete";

export const WIKILINK = /\[\[([^\]\n]+)\]\]/g;
const OPEN_WIKILINK = /\[\[[^\]\n|#]*$/;

export function wikilinkAt(text: string, offset: number): string | null {
  for (const match of text.matchAll(WIKILINK)) {
    const start = match.index;
    if (offset >= start && offset <= start + match[0].length) {
      const target = (match[1] ?? "").split("|")[0]?.split("#")[0]?.trim();
      return target || null;
    }
  }
  return null;
}

export function completeNote(
  context: CompletionContext,
  notes: string[],
): CompletionResult | null {
  const match = context.matchBefore(OPEN_WIKILINK);
  if (!match) {
    return null;
  }
  return {
    from: match.from + 2,
    validFor: /^[^\]\n|#]*$/,
    options: notes.map(noteOption),
  };
}

function noteOption(path: string): Completion {
  const title = path.slice(path.lastIndexOf("/") + 1).replace(/\.md$/i, "");
  const folder = path.includes("/")
    ? path.slice(0, path.lastIndexOf("/"))
    : undefined;
  return {
    label: title,
    detail: folder,
    apply: (editor, _completion, from, to) => {
      const closed = editor.state.sliceDoc(to, to + 2) === "]]";
      const insert = closed ? title : `${title}]]`;
      editor.dispatch({
        changes: { from, to, insert },
        selection: { anchor: from + insert.length + (closed ? 2 : 0) },
      });
    },
  };
}
