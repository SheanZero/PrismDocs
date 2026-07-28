import { describe, expect, it, vi, beforeEach } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

import { devPing } from "./ipc";

describe("devPing", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("invokes the dev_ping command", async () => {
    invoke.mockResolvedValue("3.53.2");
    await devPing();
    expect(invoke).toHaveBeenCalledWith("dev_ping");
  });

  it("returns the engine string unchanged", async () => {
    invoke.mockResolvedValue("3.53.2");
    await expect(devPing()).resolves.toBe("3.53.2");
  });

  it("propagates command errors instead of swallowing them", async () => {
    invoke.mockRejectedValue("sqlite error: disk I/O error");
    await expect(devPing()).rejects.toBe("sqlite error: disk I/O error");
  });
});
