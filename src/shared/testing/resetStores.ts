import { useEditor } from "../../notes/store/editorStore";
import { useImages } from "../../notes/store/imageStore";
import { usePropertyCatalog } from "../../notes/store/propertyStore";
import { useVault } from "../../notes/store/vaultStore";
import { usePreferences } from "../../preferences/store/preferencesStore";
import { useSync } from "../../sync/store/syncStore";
import { usePlatform } from "../../workspace/platformStore";
import { useNotifications } from "../ui/notificationStore";

export function resetStores() {
  useEditor.getState().reset();
  useVault.setState(useVault.getInitialState(), true);
  usePropertyCatalog.setState(usePropertyCatalog.getInitialState(), true);
  useImages.setState(useImages.getInitialState(), true);
  usePreferences.setState(usePreferences.getInitialState(), true);
  useNotifications.setState(useNotifications.getInitialState(), true);
  useSync.setState(useSync.getInitialState(), true);
  usePlatform.setState(usePlatform.getInitialState(), true);
}
