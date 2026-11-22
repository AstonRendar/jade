import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { SearchHitView } from "../../shared/bindings/SearchHitView";
import { Palette } from "../../shared/ui/Palette";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { notesGateway } from "../gateway/notesGateway";
import { useVault } from "../store/vaultStore";

const SEARCH_DELAY_MS = 200;

export function SearchDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHitView[]>([]);

  useEffect(() => {
    if (query.trim() === "") {
      return;
    }
    const timer = setTimeout(() => {
      void reportingErrors(async () =>
        setHits(await notesGateway.search(query)),
      );
    }, SEARCH_DELAY_MS);
    return () => clearTimeout(timer);
  }, [query]);

  const visibleHits = query.trim() === "" ? [] : hits;
  return (
    <Palette
      label={t("navigation.search")}
      placeholder={t("navigation.searchPlaceholder")}
      query={query}
      options={visibleHits.map((hit) => ({
        id: `${hit.path}:${hit.line}`,
        label: hit.text,
        detail: hit.line === 0 ? hit.path : `${hit.path}:${hit.line}`,
      }))}
      onQuery={setQuery}
      onPick={(option) => {
        onClose();
        useVault
          .getState()
          .select(option.id.slice(0, option.id.lastIndexOf(":")));
      }}
      onClose={onClose}
    />
  );
}
