import type { Locale } from "./resources";

const FALLBACK_LOCALE: Locale = "en";
const SIMPLIFIED_CHINESE = /^zh(-(cn|sg|hans)(-.*)?)?$/;
const LANGUAGE_ONLY_LOCALES: readonly Locale[] = ["en", "es", "ca"];

export function resolveLocale(preferred: readonly string[]): Locale {
  for (const tag of preferred) {
    const locale = toSupportedLocale(normalize(tag));
    if (locale) {
      return locale;
    }
  }
  return FALLBACK_LOCALE;
}

function normalize(tag: string): string {
  return tag.toLowerCase().replace(/_/g, "-");
}

function toSupportedLocale(tag: string): Locale | undefined {
  if (SIMPLIFIED_CHINESE.test(tag)) {
    return "zh-CN";
  }
  const language = tag.split("-")[0];
  return LANGUAGE_ONLY_LOCALES.find((locale) => locale === language);
}
