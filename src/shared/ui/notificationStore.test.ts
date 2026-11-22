import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { resetStores } from "../testing/resetStores";
import {
  ERROR_LIFETIME_MS,
  INFO_LIFETIME_MS,
  useNotifications,
} from "./notificationStore";

const networkError = { code: "network", params: { detail: "offline" } };

function shown() {
  return useNotifications.getState().notifications;
}

describe("notifications", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    resetStores();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("dismisses an error on its own after a long while", () => {
    // Arrange
    useNotifications.getState().report(networkError);

    // Act
    vi.advanceTimersByTime(ERROR_LIFETIME_MS - 1);
    const beforeExpiry = shown().length;
    vi.advanceTimersByTime(1);

    // Assert
    expect(beforeExpiry).toBe(1);
    expect(shown()).toEqual([]);
  });

  it("dismisses an information notice sooner than an error", () => {
    // Arrange
    useNotifications.getState().inform("sync.finished");

    // Act
    vi.advanceTimersByTime(INFO_LIFETIME_MS);

    // Assert
    expect(INFO_LIFETIME_MS).toBeLessThan(ERROR_LIFETIME_MS);
    expect(shown()).toEqual([]);
  });

  it("flashes the notice already shown instead of repeating it", () => {
    // Arrange
    useNotifications.getState().report(networkError);
    const [first] = shown();

    // Act
    useNotifications.getState().report(networkError);

    // Assert
    expect(shown()).toEqual([{ ...first, flashes: 1 }]);
  });

  it("keeps a repeated notice on screen for a full lifetime again", () => {
    // Arrange
    useNotifications.getState().report(networkError);
    vi.advanceTimersByTime(ERROR_LIFETIME_MS - 1000);

    // Act
    useNotifications.getState().report(networkError);
    vi.advanceTimersByTime(ERROR_LIFETIME_MS - 1);

    // Assert
    expect(shown()).toHaveLength(1);
  });

  it("shows different notices side by side", () => {
    // Arrange
    useNotifications.getState().report(networkError);

    // Act
    useNotifications
      .getState()
      .report({ code: "network", params: { detail: "timeout" } });

    // Assert
    expect(shown()).toHaveLength(2);
  });
});
