import { useEditor } from "../../notes/store/editorStore";
import { useVault } from "../../notes/store/vaultStore";
import { usePreferences } from "../../preferences/store/preferencesStore";
import { useNotifications } from "../ui/notificationStore";

export function resetStores() {
  useEditor.getState().reset();
  useVault.setState(useVault.getInitialState(), true);
  usePreferences.setState(usePreferences.getInitialState(), true);
  useNotifications.setState(useNotifications.getInitialState(), true);
}
