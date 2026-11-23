import { create } from "zustand";
import type { PropertySummaryView } from "../../shared/bindings/PropertySummaryView";
import { notesGateway } from "../gateway/notesGateway";

type PropertyCatalogState = {
  catalog: PropertySummaryView[];
  load: () => Promise<void>;
};

export const usePropertyCatalog = create<PropertyCatalogState>((set) => ({
  catalog: [],
  load: async () => set({ catalog: await notesGateway.listProperties() }),
}));
