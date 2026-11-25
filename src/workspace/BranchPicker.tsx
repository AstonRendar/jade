import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useEditor } from "../notes/store/editorStore";
import { useVault } from "../notes/store/vaultStore";
import type { BranchListView } from "../shared/bindings/BranchListView";
import { BusyOverlay } from "../shared/ui/BusyOverlay";
import { reportingErrors } from "../shared/ui/notificationStore";
import { Palette } from "../shared/ui/Palette";
import { syncGateway } from "../sync/gateway/syncGateway";
import { useSync } from "../sync/store/syncStore";

export function BranchPicker({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [list, setList] = useState<BranchListView | null>(null);
  const [query, setQuery] = useState("");
  const [switching, setSwitching] = useState<string | null>(null);

  useEffect(() => {
    void reportingErrors(async () => setList(await syncGateway.listBranches()));
  }, []);

  const needle = query.trim().toLocaleLowerCase();
  const options = (list?.branches ?? [])
    .filter((branch) => branch.toLocaleLowerCase().includes(needle))
    .map((branch) => ({
      id: branch,
      label: branch,
      detail: branch === list?.current ? t("sync.currentBranch") : undefined,
    }));

  if (switching !== null) {
    return (
      <BusyOverlay
        message={t("sync.switchingBranch", { branch: switching })}
        detail={t("sync.switchingBranchDetail")}
      />
    );
  }

  return (
    <Palette
      label={t("sync.chooseBranch")}
      placeholder={t("sync.branchPlaceholder")}
      query={query}
      options={options}
      onQuery={setQuery}
      onPick={(option) => {
        if (option.id === list?.current) {
          onClose();
          return;
        }
        setSwitching(option.id);
        void reportingErrors(() => switchTo(option.id)).finally(onClose);
      }}
      onClose={onClose}
    />
  );
}

async function switchTo(branch: string) {
  await useEditor.getState().flush();
  await syncGateway.switchBranch(branch);
  const selected = useVault.getState().selected;
  await useVault.getState().refresh();
  const stillThere = useVault
    .getState()
    .entries.some((entry) => entry.path === selected);
  if (selected !== null && stillThere) {
    await useEditor.getState().load(selected);
  } else if (selected !== null) {
    useEditor.getState().reset();
    useVault.setState({ selected: null });
  }
  await useSync.getState().refresh();
}
