import { create } from "zustand";
import { notesGateway } from "../gateway/notesGateway";
import { isExternal } from "../images/imageFiles";
import { useVault } from "./vaultStore";

type ImageState = {
  urls: Record<string, string | null>;
  resolve: (targets: string[]) => Promise<void>;
  refresh: () => Promise<void>;
  forget: () => void;
};

export const useImages = create<ImageState>((set, get) => ({
  urls: {},
  resolve: async (targets) => {
    const known = get().urls;
    await lookUp(
      [...new Set(targets)].filter((target) => !(target in known)),
      set,
    );
  },
  refresh: async () => {
    await lookUp(Object.keys(get().urls), set);
  },
  forget: () => set({ urls: {} }),
}));

async function lookUp(
  targets: string[],
  set: (update: (state: ImageState) => Partial<ImageState>) => void,
): Promise<void> {
  const vault = useVault.getState().vault;
  if (vault === null || targets.length === 0) {
    return;
  }
  const external = targets.filter(isExternal);
  const local = targets.filter((target) => !isExternal(target));
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
}
