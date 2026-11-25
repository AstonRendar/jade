import type {
  MarkdownIt,
  StateBlock,
  StateCore,
  StateInline,
  Token,
} from "markdown-it";

const EQUALS = 0x3d;
const PERCENT = 0x25;
const HASH = 0x23;
const TAG_CHARACTER = /[\p{L}\p{N}_\-/]/u;
const CALLOUT_HEADER = /^\[!([\w-]+)\]([+-]?)[ \t]*(.*)$/;
const TASK_MARKER = /^\[([ xX])\]\s+/;

export const CALLOUT_ICONS: Record<string, string> = {
  note: "✎",
  abstract: "≡",
  info: "ℹ",
  todo: "☐",
  tip: "✦",
  success: "✓",
  question: "?",
  warning: "⚠",
  failure: "✕",
  danger: "⚡",
  bug: "✱",
  example: "◆",
  quote: "❝",
};

const CALLOUT_ALIASES: Record<string, string> = {
  summary: "abstract",
  tldr: "abstract",
  hint: "tip",
  important: "tip",
  check: "success",
  done: "success",
  help: "question",
  faq: "question",
  caution: "warning",
  attention: "warning",
  fail: "failure",
  missing: "failure",
  error: "danger",
  cite: "quote",
};

type Callout = { kind: string; title: string; fold: "" | "+" | "-" };

export function installRules(markdown: MarkdownIt): void {
  markdown.inline.ruler.before("emphasis", "highlight", highlight);
  markdown.inline.ruler.before("emphasis", "comment", inlineComment);
  markdown.inline.ruler.push("tag", tag);
  markdown.block.ruler.before("paragraph", "comment_block", blockComment, {
    alt: ["paragraph"],
  });
  markdown.core.ruler.push("callouts", callouts);
  markdown.core.ruler.push("task_lists", taskLists);

  const escape = markdown.utils.escapeHtml;
  markdown.renderer.rules.highlight_open = () => "<mark>";
  markdown.renderer.rules.highlight_close = () => "</mark>";
  markdown.renderer.rules.tag = (tokens, index) => {
    const name = tokens[index]?.content ?? "";
    return `<span class="tag">#${escape(name)}</span>`;
  };
  markdown.renderer.rules.task_checkbox = (tokens, index) => {
    const checked = tokens[index]?.meta?.checked === true;
    return `<input class="task" type="checkbox" disabled${checked ? " checked" : ""}> `;
  };
  markdown.renderer.rules.blockquote_open = (
    tokens,
    index,
    options,
    _env,
    self,
  ) => {
    const callout = tokens[index]?.meta as Callout | undefined;
    if (!callout) {
      return self.renderToken(tokens, index, options);
    }
    const icon = CALLOUT_ICONS[callout.kind] ?? CALLOUT_ICONS.note;
    const title = `<span class="callout-icon" aria-hidden="true">${icon}</span><span class="callout-title-text">${escape(callout.title)}</span>`;
    if (callout.fold) {
      const open = callout.fold === "+" ? " open" : "";
      return `<details class="callout" data-callout="${escape(callout.kind)}"${open}><summary class="callout-title">${title}</summary><div class="callout-content">\n`;
    }
    return `<div class="callout" data-callout="${escape(callout.kind)}"><div class="callout-title">${title}</div><div class="callout-content">\n`;
  };
  markdown.renderer.rules.blockquote_close = (
    tokens,
    index,
    options,
    _env,
    self,
  ) => {
    const callout = tokens[index]?.meta as Callout | undefined;
    if (!callout) {
      return self.renderToken(tokens, index, options);
    }
    return callout.fold ? "</div></details>\n" : "</div></div>\n";
  };
  markdown.renderer.rules.table_open = () => '<div class="table"><table>\n';
  markdown.renderer.rules.table_close = () => "</table></div>\n";
  const fence = markdown.renderer.rules.fence;
  markdown.renderer.rules.fence = (tokens, index, options, env, self) => {
    const language = tokens[index]?.info.trim().split(/\s+/)[0] ?? "";
    const code = fence ? fence(tokens, index, options, env, self) : "";
    if (!language) {
      return code;
    }
    return `<div class="code-block"><span class="code-language">${escape(language)}</span>${code}</div>`;
  };
}

function highlight(state: StateInline, silent: boolean): boolean {
  const start = state.pos;
  if (
    state.src.charCodeAt(start) !== EQUALS ||
    state.src.charCodeAt(start + 1) !== EQUALS
  ) {
    return false;
  }
  const end = state.src.indexOf("==", start + 2);
  if (
    end === -1 ||
    end === start + 2 ||
    /\s/.test(state.src[start + 2] ?? "")
  ) {
    return false;
  }
  if (!silent) {
    state.push("highlight_open", "mark", 1);
    const oldMax = state.posMax;
    state.pos = start + 2;
    state.posMax = end;
    state.md.inline.tokenize(state);
    state.posMax = oldMax;
    state.push("highlight_close", "mark", -1);
  }
  state.pos = end + 2;
  return true;
}

function inlineComment(state: StateInline, silent: boolean): boolean {
  const start = state.pos;
  if (
    state.src.charCodeAt(start) !== PERCENT ||
    state.src.charCodeAt(start + 1) !== PERCENT
  ) {
    return false;
  }
  const end = state.src.indexOf("%%", start + 2);
  if (end === -1) {
    return false;
  }
  if (!silent) {
    state.pending = state.pending.trimEnd();
  }
  state.pos = end + 2;
  return true;
}

function blockComment(
  state: StateBlock,
  startLine: number,
  endLine: number,
  silent: boolean,
): boolean {
  const lineText = (line: number) =>
    state.src.slice(
      (state.bMarks[line] ?? 0) + (state.tShift[line] ?? 0),
      state.eMarks[line],
    );
  if (!lineText(startLine).startsWith("%%")) {
    return false;
  }
  let line = startLine;
  let text = lineText(line).slice(2);
  while (!text.includes("%%")) {
    line += 1;
    if (line >= endLine) {
      return false;
    }
    text = lineText(line);
  }
  if (!silent) {
    state.line = line + 1;
  }
  return true;
}

function tag(state: StateInline, silent: boolean): boolean {
  const start = state.pos;
  if (state.src.charCodeAt(start) !== HASH) {
    return false;
  }
  const previous = state.src[start - 1];
  if (previous !== undefined && !/\s/.test(previous)) {
    return false;
  }
  let end = start + 1;
  while (end < state.posMax && TAG_CHARACTER.test(state.src[end] ?? "")) {
    end += 1;
  }
  const name = state.src.slice(start + 1, end).replace(/\/+$/, "");
  if (name === "" || /^[\d/]+$/.test(name)) {
    return false;
  }
  if (!silent) {
    const token = state.push("tag", "span", 0);
    token.content = name;
  }
  state.pos = start + 1 + name.length;
  return true;
}

function callouts(state: StateCore): void {
  const tokens = state.tokens;
  tokens.forEach((token, index) => {
    if (token.type !== "blockquote_open") {
      return;
    }
    const inline = tokens[index + 2];
    if (
      tokens[index + 1]?.type !== "paragraph_open" ||
      inline?.type !== "inline"
    ) {
      return;
    }
    const [header = "", ...rest] = inline.content.split("\n");
    const match = CALLOUT_HEADER.exec(header);
    if (!match) {
      return;
    }
    const raw = (match[1] ?? "note").toLowerCase();
    const kind = CALLOUT_ALIASES[raw] ?? (raw in CALLOUT_ICONS ? raw : "note");
    const callout: Callout = {
      kind,
      fold: (match[2] ?? "") as Callout["fold"],
      title: match[3]?.trim() || raw.charAt(0).toUpperCase() + raw.slice(1),
    };
    token.meta = callout;
    closingQuote(tokens, index).meta = callout;
    dropHeaderLine(inline, rest, tokens, index);
  });
}

function closingQuote(tokens: Token[], openIndex: number): Token {
  let depth = 0;
  for (let index = openIndex; index < tokens.length; index += 1) {
    const type = tokens[index]?.type;
    if (type === "blockquote_open") {
      depth += 1;
    } else if (type === "blockquote_close") {
      depth -= 1;
      if (depth === 0) {
        return tokens[index] as Token;
      }
    }
  }
  return tokens[openIndex] as Token;
}

function dropHeaderLine(
  inline: Token,
  rest: string[],
  tokens: Token[],
  index: number,
): void {
  const children = inline.children ?? [];
  const breakAt = children.findIndex(
    (child) => child.type === "softbreak" || child.type === "hardbreak",
  );
  if (rest.length === 0 || breakAt === -1) {
    tokens.splice(index + 1, 3);
    return;
  }
  inline.content = rest.join("\n");
  inline.children = children.slice(breakAt + 1);
}

function taskLists(state: StateCore): void {
  const tokens = state.tokens;
  tokens.forEach((token, index) => {
    if (
      token.type !== "inline" ||
      tokens[index - 2]?.type !== "list_item_open"
    ) {
      return;
    }
    const first = token.children?.[0];
    const match =
      first?.type === "text" ? TASK_MARKER.exec(first.content) : null;
    if (!first || !match) {
      return;
    }
    first.content = first.content.slice(match[0].length);
    const checkbox = new state.Token("task_checkbox", "input", 0);
    checkbox.meta = { checked: match[1] !== " " };
    token.children?.unshift(checkbox);
    const item = tokens[index - 2] as Token;
    item.attrJoin("class", match[1] === " " ? "task-item" : "task-item done");
  });
}
