import i18next, { type i18n } from "i18next";
import { initReactI18next } from "react-i18next";
import { resources, type Locale } from "./resources";

export { resolveLocale } from "./resolveLocale";
export type { Locale } from "./resources";

export async function createI18n(locale: Locale): Promise<i18n> {
  const instance = i18next.createInstance();
  await instance.use(initReactI18next).init({
    resources,
    lng: locale,
    fallbackLng: "en",
    interpolation: { escapeValue: false },
  });
  return instance;
}
