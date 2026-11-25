const IMAGE_EXTENSION = /\.(png|jpe?g|gif|bmp|svg|webp|avif)$/i;
const EMBED = /!\[\[([^\]|#]+)(?:\|([^\]]*))?\]\]/g;
const MARKDOWN_IMAGE = /!\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;

export type ImageEmbed = { target: string; width?: number; height?: number };

export function isImage(path: string): boolean {
  return IMAGE_EXTENSION.test(path);
}

export function isExternal(source: string): boolean {
  return /^(https?:|data:)/i.test(source);
}

export function imageSize(
  alias: string | undefined,
): Pick<ImageEmbed, "width" | "height"> {
  const match = /^\s*(\d+)(?:x(\d+))?\s*$/.exec(alias ?? "");
  if (!match) {
    return {};
  }
  return {
    width: Number(match[1]),
    height: match[2] === undefined ? undefined : Number(match[2]),
  };
}

export function imageEmbedsIn(line: string): ImageEmbed[] {
  const embeds: ImageEmbed[] = [];
  for (const match of line.matchAll(EMBED)) {
    const target = match[1]?.trim() ?? "";
    if (isImage(target)) {
      embeds.push({ target, ...imageSize(match[2]) });
    }
  }
  for (const match of line.matchAll(MARKDOWN_IMAGE)) {
    const source = match[1] ?? "";
    if (isExternal(source)) {
      embeds.push({ target: source });
    } else if (isImage(source)) {
      embeds.push({ target: safeDecode(source) });
    }
  }
  return embeds;
}

export function safeDecode(source: string): string {
  try {
    return decodeURI(source);
  } catch {
    return source;
  }
}

export function pastedImageName(type: string, now: Date): string {
  const extension = type.split("/")[1]?.replace("jpeg", "jpg") || "png";
  const pad = (value: number) => String(value).padStart(2, "0");
  const stamp = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
  return `Pasted image ${stamp}.${extension}`;
}
