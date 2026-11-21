import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { createI18n, resolveLocale } from "./shared/i18n";
import "./shared/ui/global.css";

const container = document.getElementById("root");
if (!container) {
  throw new Error("Root element not found");
}

createI18n(resolveLocale(navigator.languages)).then((i18n) => {
  createRoot(container).render(
    <StrictMode>
      <App i18n={i18n} />
    </StrictMode>,
  );
});
