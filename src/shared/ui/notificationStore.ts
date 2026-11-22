import { create } from "zustand";
import type { ErrorView } from "../bindings/ErrorView";
import { isErrorView } from "../gateway/call";

export type Notification = {
  id: number;
  kind: "error" | "info";
  key: string;
  params: Record<string, string>;
};

type NotificationsState = {
  notifications: Notification[];
  report: (error: unknown) => void;
  inform: (key: string, params?: Record<string, string>) => void;
  dismiss: (id: number) => void;
};

let nextId = 1;

export const useNotifications = create<NotificationsState>((set) => {
  const push = (notification: Omit<Notification, "id">) =>
    set((state) => ({
      notifications: [
        ...state.notifications,
        { id: nextId++, ...notification },
      ],
    }));
  return {
    notifications: [],
    report: (error) => {
      const view: ErrorView = isErrorView(error)
        ? error
        : { code: "unexpected", params: { detail: String(error) } };
      push({ kind: "error", key: `errors.${view.code}`, params: view.params });
    },
    inform: (key, params = {}) => push({ kind: "info", key, params }),
    dismiss: (id) =>
      set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== id),
      })),
  };
});

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
