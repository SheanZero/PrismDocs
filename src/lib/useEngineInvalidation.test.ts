import { beforeEach, describe, expect, it, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { act, renderHook } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

type Handler = (event: { payload: unknown }) => void;

// 工厂体内只**引用**而不解引用 listenSpy —— vi.mock 会被提升到静态 import 之上，
// 提前解引用会撞上 TDZ（沿用 ipc.test.ts 已验证的写法）。
const listenSpy = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: Handler) => listenSpy(name, handler),
}));

import { EVENT_CHANGED, type EngineEvent } from "./ipc";
import { useEngineInvalidation } from "./useEngineInvalidation";

/// 当前挂着的 listener。它的 **size** 是 Pitfall 5 的直接观测量：
/// 漏掉 cleanup 时它会随挂载次数增长，而"失效次数"只是那个增长的下游表现。
const handlers = new Map<number, Handler>();
let nextId = 0;

function emit(payload: EngineEvent) {
  for (const handler of [...handlers.values()]) handler({ payload });
}

/// 冲掉 `listen` 的 Promise 与 cleanup 里的 `.then(un => un())`（各占一个微任务）。
async function flush() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

function setup() {
  const client = new QueryClient();
  const invalidate = vi
    .spyOn(client, "invalidateQueries")
    .mockResolvedValue(undefined);
  const wrapper = ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client }, children);
  return { invalidate, wrapper };
}

beforeEach(() => {
  handlers.clear();
  nextId = 0;
  listenSpy.mockReset();
  listenSpy.mockImplementation(async (_name: string, handler: Handler) => {
    const id = nextId++;
    handlers.set(id, handler);
    return () => {
      handlers.delete(id);
    };
  });
});

describe("useEngineInvalidation", () => {
  it("subscribes to the coarse invalidation event name", async () => {
    const { wrapper } = setup();
    renderHook(() => useEngineInvalidation(), { wrapper });
    await flush();

    expect(listenSpy).toHaveBeenCalledWith(EVENT_CHANGED, expect.any(Function));
    expect(handlers.size).toBe(1);
  });

  it("invalidates only the affected project on docChanged", async () => {
    const { invalidate, wrapper } = setup();
    renderHook(() => useEngineInvalidation(), { wrapper });
    await flush();

    emit({ kind: "docChanged", projectId: "p1", docId: "d1" });

    expect(invalidate).toHaveBeenCalledTimes(1);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ["docs", "p1"] });
  });

  // Resync 的语义是「你手上的东西全都作废」：broadcast Lagged 之后丢了多少条、
  // 丢的是哪些 project 都不可知，按 key 失效等于替不可知的事实做了一个假设。
  it("invalidates everything on resync, with no query key at all", async () => {
    const { invalidate, wrapper } = setup();
    renderHook(() => useEngineInvalidation(), { wrapper });
    await flush();

    emit({ kind: "resync" });

    expect(invalidate).toHaveBeenCalledTimes(1);
    expect(invalidate.mock.calls[0]).toHaveLength(0);
  });

  it("stops invalidating after unmount", async () => {
    const { invalidate, wrapper } = setup();
    const view = renderHook(() => useEngineInvalidation(), { wrapper });
    await flush();

    view.unmount();
    await flush();
    expect(handlers.size).toBe(0);

    emit({ kind: "docChanged", projectId: "p1", docId: "d1" });
    expect(invalidate).not.toHaveBeenCalled();
  });

  // **判别性的那一条（Pitfall 5）。** React 19 StrictMode 下 effect 会执行两次，
  // 这里用「挂载 → 卸载 → 挂载」确定性地复现同一个序列（不依赖 React 是否为 dev 构建，
  // 否则测试可能在 prod 构建下静默退化成单次挂载而恒绿）。
  // 漏掉 cleanup 时 handlers.size 变 2，一条事件触发两次失效 —— 两条断言各自指名一个事实。
  it("registers exactly one listener across a StrictMode-style double effect", async () => {
    const { invalidate, wrapper } = setup();

    const first = renderHook(() => useEngineInvalidation(), { wrapper });
    await flush();
    first.unmount();
    await flush();
    renderHook(() => useEngineInvalidation(), { wrapper });
    await flush();

    expect(handlers.size).toBe(1);

    emit({ kind: "docChanged", projectId: "p1", docId: "d1" });
    expect(invalidate).toHaveBeenCalledTimes(1);
  });
});
