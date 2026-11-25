import MarkdownIt from "markdown-it";
import {
  imageSize,
  isExternal,
  isImage,
  safeDecode,
} from "../images/imageFiles";
import { installRules } from "./markdownRules";

const markdown = new MarkdownIt({ html: false, linkify: true });
installRules(markdown);

markdown.inline.ruler.before("link", "wikilink", (state, silent) => {
  const embed = state.src.startsWith("![[", state.pos);
  const start = embed ? state.pos + 1 : state.pos;
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
  if (!silent && embed && isImage(link.target)) {
    const [, alias] = state.src.slice(start + 2, end).split("|");
    const token = state.push("image_embed", "img", 0);
    token.meta = { target: link.target, ...imageSize(alias) };
  } else if (!silent) {
    const token = state.push("wikilink", "", 0);
    token.meta = { ...link, embed };
  }
  state.pos = end + 2;
  return true;
});

markdown.renderer.rules.wikilink = (tokens, index) => {
  const { target, label, embed } = tokens[index]?.meta as Wikilink & {
    embed: boolean;
  };
  const escape = markdown.utils.escapeHtml;
  const kind = embed ? "wikilink embed" : "wikilink";
  return `<a class="${kind}" href="#" data-target="${escape(target)}">${escape(label)}</a>`;
};

type Wikilink = { target: string; label: string };
type Images = Record<string, string | null>;
type ImageEnv = { images?: Images };

markdown.renderer.rules.image_embed = (tokens, index, _options, env) => {
  const { target, width, height } = tokens[index]?.meta as {
    target: string;
    width?: number;
    height?: number;
  };
  return imageTag(
    target,
    (env as ImageEnv | undefined)?.images,
    "",
    width,
    height,
  );
};

markdown.renderer.rules.image = (tokens, index, _options, env) => {
  const token = tokens[index];
  const source = String(token?.attrGet("src") ?? "");
  const alt = token?.children?.map((child) => child.content).join("") ?? "";
  const target = isExternal(source) ? source : safeDecode(source);
  return imageTag(target, (env as ImageEnv | undefined)?.images, alt);
};

function imageTag(
  target: string,
  images: Images | undefined,
  alt: string,
  width?: number,
  height?: number,
): string {
  const escape = markdown.utils.escapeHtml;
  const url = isExternal(target) ? target : images?.[target];
  if (!url) {
    return `<span class="embed-image missing" data-image="${escape(target)}">${escape(target)}</span>`;
  }
  const size = `${width ? ` width="${width}"` : ""}${height ? ` height="${height}"` : ""}`;
  return `<img class="embed-image" src="${escape(url)}" alt="${escape(alt)}" data-image="${escape(target)}"${size} loading="lazy">`;
}

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

export function renderMarkdown(source: string, images?: Images): string {
  return markdown.render(source, { images });
}
