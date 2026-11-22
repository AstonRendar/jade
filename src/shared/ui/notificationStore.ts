import { create } from "zustand";
import type { ErrorView } from "../bindings/ErrorView";
import { isErrorView } from "../gateway/call";

type Notification = { id: number; error: ErrorView };

type NotificationsState = {
  notifications: Notification[];
  report: (error: unknown) => void;
  dismiss: (id: number) => void;
};

let nextId = 1;

export const useNotifications = create<NotificationsState>((set) => ({
  notifications: [],
  report: (error) => {
    const view: ErrorView = isErrorView(error)
      ? error
      : { code: "unexpected", params: { detail: String(error) } };
    set((state) => ({
      notifications: [...state.notifications, { id: nextId++, error: view }],
    }));
  },
  dismiss: (id) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),
}));

export async function reportingErrors<T>(
  action: () => Promise<T>,
): Promise<T | undefined> {
  try {
    return await action();
  } catch (error) {
    useNotifications.getState().report(error);
    return undefined;
  }
}
