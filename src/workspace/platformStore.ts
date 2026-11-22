import { create } from "zustand";
import type { CapabilitiesView } from "../shared/bindings/CapabilitiesView";
import { platformGateway } from "../shared/gateway/platformGateway";

type PlatformState = {
  capabilities: CapabilitiesView | null;
  load: () => Promise<void>;
};

export const usePlatform = create<PlatformState>((set) => ({
  capabilities: null,
  load: async () => set({ capabilities: await platformGateway.capabilities() }),
}));
