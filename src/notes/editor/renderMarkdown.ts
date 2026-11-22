import MarkdownIt from "markdown-it";

const markdown = new MarkdownIt({ html: false, linkify: true });

markdown.inline.ruler.before("link", "wikilink", (state, silent) => {
  const start = state.pos;
  if (state.src.slice(start, start + 2) !== "[[") {
    return false;
  }
  const end = state.src.indexOf("]]", start + 2);
  if (end === -1) {
    return false;
  }
  const link = parseWikilink(state.src.slice(start + 2, end));
  if (link === null) {
    return false;
  }
  if (!silent) {
    const token = state.push("wikilink", "", 0);
    token.meta = link;
  }
  state.pos = end + 2;
  return true;
});

markdown.renderer.rules.wikilink = (tokens, index) => {
  const { target, label } = tokens[index]?.meta as Wikilink;
  const escape = markdown.utils.escapeHtml;
  return `<a class="wikilink" href="#" data-target="${escape(target)}">${escape(label)}</a>`;
};

type Wikilink = { target: string; label: string };

function parseWikilink(inner: string): Wikilink | null {
  const [reference = "", alias] = inner.split("|");
  const [target = "", heading] = reference.split("#");
  const cleanTarget = target.trim();
  if (cleanTarget === "") {
    return null;
  }
  const fallback = heading ? `${cleanTarget} › ${heading.trim()}` : cleanTarget;
  return { target: cleanTarget, label: alias?.trim() || fallback };
}

export function renderMarkdown(source: string): string {
  return markdown.render(source);
}
