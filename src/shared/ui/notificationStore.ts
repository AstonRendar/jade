import { create } from "zustand";
import type { ErrorView } from "../bindings/ErrorView";
import { isErrorView } from "../gateway/call";

export type Notification = {
  id: number;
  kind: "error" | "info";
  key: string;
  params: Record<string, string>;
  flashes: number;
};

export const ERROR_LIFETIME_MS = 20_000;
export const INFO_LIFETIME_MS = 6_000;

type NotificationsState = {
  notifications: Notification[];
  report: (error: unknown) => void;
  inform: (key: string, params?: Record<string, string>) => void;
  dismiss: (id: number) => void;
};

type Notice = Omit<Notification, "id" | "flashes">;

let nextId = 1;
const expirations = new Map<number, ReturnType<typeof setTimeout>>();

function sameNotice(notification: Notification, notice: Notice): boolean {
  return (
    notification.kind === notice.kind &&
    notification.key === notice.key &&
    JSON.stringify(notification.params) === JSON.stringify(notice.params)
  );
}

export const useNotifications = create<NotificationsState>((set, get) => {
  const expireLater = (id: number, kind: Notification["kind"]) => {
    clearTimeout(expirations.get(id));
    const lifetime = kind === "error" ? ERROR_LIFETIME_MS : INFO_LIFETIME_MS;
    expirations.set(
      id,
      setTimeout(() => get().dismiss(id), lifetime),
    );
  };
  const push = (notice: Notice) => {
    const shown = get().notifications.find((n) => sameNotice(n, notice));
    const id = shown?.id ?? nextId++;
    set((state) => ({
      notifications: shown
        ? state.notifications.map((n) =>
            n.id === id ? { ...n, flashes: n.flashes + 1 } : n,
          )
        : [...state.notifications, { id, flashes: 0, ...notice }],
    }));
    expireLater(id, notice.kind);
  };
  return {
    notifications: [],
    report: (error) => {
      const view: ErrorView = isErrorView(error)
        ? error
        : { code: "unexpected", params: { detail: String(error) } };
      push({ kind: "error", key: `errors.${view.code}`, params: view.params });
    },
    inform: (key, params = {}) => push({ kind: "info", key, params }),
    dismiss: (id) => {
      clearTimeout(expirations.get(id));
      expirations.delete(id);
      set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== id),
      }));
    },
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
