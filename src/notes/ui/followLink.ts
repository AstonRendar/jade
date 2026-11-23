import { reportingErrors } from "../../shared/ui/notificationStore";
import { notesGateway } from "../gateway/notesGateway";
import { useEditor } from "../store/editorStore";
import { useVault } from "../store/vaultStore";

export function followLink(target: string) {
  void reportingErrors(async () => {
    await useEditor.getState().flush();
    const followed = await notesGateway.followLink(target);
    if (followed.created) {
      await useVault.getState().refresh();
    }
    useVault.getState().select(followed.path);
  });
}
