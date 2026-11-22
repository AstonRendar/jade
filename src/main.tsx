import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createI18n, resolveLocale } from "./shared/i18n";
import "./shared/ui/global.css";
import { Workspace } from "./workspace/Workspace";

const container = document.getElementById("root");
if (!container) {
  throw new Error("Root element not found");
}

createI18n(resolveLocale(navigator.languages)).then((i18n) => {
  createRoot(container).render(
    <StrictMode>
      <Workspace i18n={i18n} />
    </StrictMode>,
  );
});
