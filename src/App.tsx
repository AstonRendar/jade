import type { i18n } from "i18next";
import { I18nextProvider, useTranslation } from "react-i18next";

export function App({ i18n }: { i18n: i18n }) {
  return (
    <I18nextProvider i18n={i18n}>
      <Welcome />
    </I18nextProvider>
  );
}

function Welcome() {
  const { t } = useTranslation();
  return (
    <main className="welcome">
      <h1>{t("welcome.title")}</h1>
      <p>{t("welcome.subtitle")}</p>
    </main>
  );
}
