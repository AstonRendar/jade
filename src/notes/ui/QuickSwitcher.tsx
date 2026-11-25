import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Palette, type PaletteOption } from "../../shared/ui/Palette";
import { useVault } from "../store/vaultStore";
import { matchingFiles } from "./SearchPanel";

const MAX_OPTIONS = 50;

export function QuickSwitcher({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const entries = useVault((state) => state.entries);
  const [query, setQuery] = useState("");
  const options = useMemo(
    () =>
      matchingNotes(
        entries
          .filter((entry) => entry.kind === "note")
          .map((entry) => entry.path),
        query,
      ),
    [entries, query],
  );
  return (
    <Palette
      label={t("navigation.quickSwitcher")}
      placeholder={t("navigation.quickSwitcherPlaceholder")}
      query={query}
      options={options}
      onQuery={setQuery}
      onPick={(option) => {
        onClose();
        useVault.getState().select(option.id);
      }}
      onClose={onClose}
    />
  );
}

export function matchingNotes(paths: string[], query: string): PaletteOption[] {
  return matchingFiles(paths, query)
    .slice(0, MAX_OPTIONS)
    .map(({ path, name }) => ({ id: path, label: name, detail: path }));
}
