import ca from "../../locales/ca.json";
import en from "../../locales/en.json";
import es from "../../locales/es.json";
import zhCN from "../../locales/zh-CN.json";

export const SUPPORTED_LOCALES = ["en", "es", "zh-CN", "ca"] as const;

export type Locale = (typeof SUPPORTED_LOCALES)[number];

export const resources = {
  en: { translation: en },
  es: { translation: es },
  "zh-CN": { translation: zhCN },
  ca: { translation: ca },
} satisfies Record<Locale, { translation: typeof en }>;
