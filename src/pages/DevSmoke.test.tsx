import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import type { SmokeEvent } from "../lib/ipc";

type Handler = (event: { payload: unknown }) => void;

const listenSpy = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: Handler) => listenSpy(name, handler),
}));

const devEmitBusEvent = vi.fn();
const devSmokeStream = vi.fn();
const searchDocuments = vi.fn();
const devSeedSampleDocs = vi.fn();

vi.mock("../lib/ipc", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/ipc")>();
  return {
    ...actual,
    devEmitBusEvent: (projectId: string, docId: string) =>
      devEmitBusEvent(projectId, docId),
    devSmokeStream: (total: number, onEvent: (ev: SmokeEvent) => void) =>
      devSmokeStream(total, onEvent),
    searchDocuments: (projectId: string, q: string) =>
      searchDocuments(projectId, q),
    devSeedSampleDocs: () => devSeedSampleDocs(),
  };
});

import DevSmokePage, { verifySmokeStream } from "./DevSmoke";

const handlers = new Map<number, Handler>();
let nextId = 0;

function emitChanged() {
  for (const handler of [...handlers.values()]) {
    handler({ payload: { kind: "docChanged", projectId: "p", docId: "d" } });
  }
}

function fullStream(total: number): SmokeEvent[] {
  const out: SmokeEvent[] = [{ event: "started", data: { total } }];
  for (let seq = 0; seq < total; seq++) out.push({ event: "tick", data: { seq } });
  out.push({ event: "finished", data: { total } });
  return out;
}

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return render(<DevSmokePage />, { wrapper });
}

// vitest 的 globals 未开启，Testing Library 的自动清理不会注册——不显式 cleanup
// 就会把上一条测试的 DOM 留在 document 里，随后 getByRole 报 "Found multiple elements"。
afterEach(cleanup);

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
  for (const spy of [devEmitBusEvent, devSmokeStream, searchDocuments, devSeedSampleDocs]) {
    spy.mockReset();
  }
  devEmitBusEvent.mockResolvedValue(undefined);
  searchDocuments.mockResolvedValue([]);
  devSeedSampleDocs.mockResolvedValue("smoke-project");
  devSmokeStream.mockImplementation(
    async (total: number, onEvent: (ev: SmokeEvent) => void) => {
      for (const ev of fullStream(total)) onEvent(ev);
    },
  );
});

describe("verifySmokeStream", () => {
  it("accepts Started + 0..total-1 ticks + Finished", () => {
    expect(verifySmokeStream(fullStream(1000), 1000)).toEqual({
      ok: true,
      ticks: 1000,
    });
  });

  it("rejects a gap and names where it happened", () => {
    const events = fullStream(4).filter(
      (ev) => !(ev.event === "tick" && ev.data.seq === 2),
    );
    const verdict = verifySmokeStream(events, 4);
    expect(verdict.ok).toBe(false);
    // 缺口位置必须出现在结论里——「失败了」不足以指路。
    expect(verdict.ok === false && verdict.reason).toContain("2");
  });

  it("rejects an out-of-order stream even though the set of seqs is complete", () => {
    const events = fullStream(3);
    const swapped = [events[0], events[2], events[1], events[3], events[4]];
    expect(verifySmokeStream(swapped as SmokeEvent[], 3).ok).toBe(false);
  });

  it("rejects a stream whose last event is not Finished", () => {
    const events = fullStream(3).slice(0, -1);
    expect(verifySmokeStream(events, 3).ok).toBe(false);
  });
});

describe("DevSmokePage", () => {
  // Pitfall 5 的可观察证据：点两次，收两条事件，计数恰好 2（不是 4，也不是 0）。
  it("counts one bus event per click, 1:1", async () => {
    renderPage();
    await waitFor(() => expect(handlers.size).toBe(1));

    const button = screen.getByRole("button", { name: /触发总线事件/ });

    fireEvent.click(button);
    await waitFor(() => expect(devEmitBusEvent).toHaveBeenCalledTimes(1));
    await act(async () => emitChanged());

    fireEvent.click(button);
    await waitFor(() => expect(devEmitBusEvent).toHaveBeenCalledTimes(2));
    await act(async () => emitChanged());

    expect(
      (await screen.findByTestId("event-count")).textContent,
    ).toBe("2");
  });

  // 真实发生过的失败：capability 缺失时 `listen()` 被 ACL 拒绝，而 `invoke` 的自有命令
  // 不过 ACL 照常成功——点按钮没有任何报错，计数停在 0。rejection 落进 `const pending = listen(...)`
  // 这个没有 .catch 的 Promise 里，整类失败就此消失。计数为 0 不足以作断言：正常状态下它也是 0。
  it("surfaces a rejected listen instead of silently sitting at zero", async () => {
    listenSpy.mockRejectedValueOnce(
      "event.listen not allowed. Permissions associated with this command: core:event:allow-listen",
    );
    renderPage();

    const notice = await screen.findByRole("status");
    expect(notice.textContent ?? "").toContain("事件通道");
    // Tauri 的原始 ACL 文本是内部细节，不得原样进 DOM（与命令错误码同一条规矩）。
    expect(notice.textContent ?? "").not.toContain("not allowed");
  });

  it("streams with total=1000 and reports the seq verdict", async () => {
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: /Channel 有序流/ }));

    await waitFor(() => expect(devSmokeStream).toHaveBeenCalledTimes(1));
    expect(devSmokeStream.mock.calls[0][0]).toBe(1000);

    const verdict = await screen.findByTestId("stream-verdict");
    expect(verdict.textContent ?? "").toContain("通过");
    expect(verdict.textContent ?? "").toContain("1000");
  });

  it("surfaces a broken stream as a failed verdict rather than a silent pass", async () => {
    devSmokeStream.mockImplementation(
      async (total: number, onEvent: (ev: SmokeEvent) => void) => {
        for (const ev of fullStream(total)) {
          if (ev.event === "tick" && ev.data.seq === 500) continue;
          onEvent(ev);
        }
      },
    );
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: /Channel 有序流/ }));

    const verdict = await screen.findByTestId("stream-verdict");
    expect(verdict.textContent ?? "").toContain("失败");
  });

  it("reports hit counts for a Chinese query and zero for the negative control", async () => {
    searchDocuments.mockResolvedValue([
      { docId: "smoke-doc-1", title: "锚定引擎设计说明", relPath: "docs/anchor.md" },
    ]);
    renderPage();

    const input = screen.getByLabelText(/搜索词/) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "锚定引擎" } });
    fireEvent.click(screen.getByRole("button", { name: /^搜索$/ }));

    await waitFor(() =>
      expect(searchDocuments).toHaveBeenCalledWith("smoke-project", "锚定引擎"),
    );
    expect((await screen.findByTestId("search-count")).textContent).toBe("1");

    searchDocuments.mockResolvedValue([]);
    fireEvent.change(input, { target: { value: "量子纠缠" } });
    fireEvent.click(screen.getByRole("button", { name: /^搜索$/ }));

    await waitFor(() =>
      expect(searchDocuments).toHaveBeenCalledWith("smoke-project", "量子纠缠"),
    );
    await waitFor(() =>
      expect(screen.getByTestId("search-count").textContent).toBe("0"),
    );
  });

  it("seeds sample documents and adopts the project id the engine returned", async () => {
    devSeedSampleDocs.mockResolvedValue("another-project");
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: /插入样例文档/ }));
    await waitFor(() => expect(devSeedSampleDocs).toHaveBeenCalledTimes(1));

    const input = screen.getByLabelText(/搜索词/) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "锚定引擎" } });
    fireEvent.click(screen.getByRole("button", { name: /^搜索$/ }));

    await waitFor(() =>
      expect(searchDocuments).toHaveBeenCalledWith("another-project", "锚定引擎"),
    );
  });
});
