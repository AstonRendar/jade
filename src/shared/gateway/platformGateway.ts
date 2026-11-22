import type { CapabilitiesView } from "../bindings/CapabilitiesView";
import { call } from "./call";

export const platformGateway = {
  capabilities: () => call<CapabilitiesView>("app_capabilities"),
};
