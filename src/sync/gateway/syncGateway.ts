import type { VaultView } from "../../shared/bindings/VaultView";
import { call } from "../../shared/gateway/call";

export const syncGateway = {
  cloneRepository: (url: string, parent: string) =>
    call<VaultView>("sync_clone_repository", { url, parent }),
};
