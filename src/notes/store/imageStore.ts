import { create } from "zustand";
import { notesGateway } from "../gateway/notesGateway";
import { isExternal } from "../images/imageFiles";
import { useVault } from "./vaultStore";

type ImageState = {
  urls: Record<string, string | null>;
  resolve: (targets: string[]) => Promise<void>;
  forget: () => void;
};

export const useImages = create<ImageState>((set, get) => ({
  urls: {},
  resolve: async (targets) => {
    const vault = useVault.getState().vault;
    const known = get().urls;
    const missing = [...new Set(targets)].filter(
      (target) => !(target in known),
    );
    if (vault === null || missing.length === 0) {
      return;
    }
    const external = missing.filter(isExternal);
    const local = missing.filter((target) => !isExternal(target));
    const resolved =
      local.length === 0 ? [] : await notesGateway.resolveFiles(local);
    set(({ urls }) => ({
      urls: {
        ...urls,
        ...Object.fromEntries(external.map((target) => [target, target])),
        ...Object.fromEntries(
          resolved.map(({ target, path }) => [
            target,
            path === null ? null : notesGateway.fileUrl(vault.path, path),
          ]),
        ),
      },
    }));
  },
  forget: () => set({ urls: {} }),
}));
