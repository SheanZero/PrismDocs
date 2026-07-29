import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// 工厂体内只**引用**而不解引用这些 spy（vi.mock 提升到静态 import 之上，提前解引用会撞 TDZ）。
const apiKeyStatus = vi.fn();
const setApiKey = vi.fn();
const deleteApiKey = vi.fn();
const getSetting = vi.fn();
const setBaseUrl = vi.fn();

vi.mock("../lib/ipc", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/ipc")>();
  return {
    ...actual,
    apiKeyStatus: () => apiKeyStatus(),
    setApiKey: (secret: string) => setApiKey(secret),
    deleteApiKey: () => deleteApiKey(),
    getSetting: (key: string) => getSetting(key),
    setBaseUrl: (url: string) => setBaseUrl(url),
  };
});

import SettingsPage from "./Settings";

/// 测试用的假密钥。刻意**不**长得像真密钥前缀，以免 check-secrets.sh 把它当成
/// 提交进仓库的明文密钥（那个扫描器不该为了迁就 fixture 而放宽）。
const FAKE_KEY = "fixture-not-a-real-credential";

function renderPage() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return render(<SettingsPage />, { wrapper });
}

// vitest 的 globals 未开启，Testing Library 的自动清理不会注册——不显式 cleanup
// 就会把上一条测试的 DOM 留在 document 里，随后 getByRole 报 "Found multiple elements"。
afterEach(cleanup);

beforeEach(() => {
  for (const spy of [apiKeyStatus, setApiKey, deleteApiKey, getSetting, setBaseUrl]) {
    spy.mockReset();
  }
  apiKeyStatus.mockResolvedValue(false);
  getSetting.mockResolvedValue(null);
  setApiKey.mockResolvedValue(undefined);
  deleteApiKey.mockResolvedValue(undefined);
  setBaseUrl.mockResolvedValue(undefined);
});

describe("SettingsPage / API key", () => {
  it("shows 未配置 when no key is stored", async () => {
    renderPage();
    expect(await screen.findByText(/未配置/)).toBeTruthy();
    expect(screen.queryByText(/^已配置$/)).toBeNull();
  });

  it("shows 已配置 when a key is stored", async () => {
    apiKeyStatus.mockResolvedValue(true);
    renderPage();
    expect(await screen.findByText(/已配置/)).toBeTruthy();
    expect(screen.queryByText(/未配置/)).toBeNull();
  });

  // T-01-04c。两条断言缺一不可：只有「页面上没有密钥」时，一个**根本没提交表单**的
  // 组件也会通过；配上「setApiKey 确实收到了这个密钥」才说明这条路真的走过。
  it("never echoes the typed key back into the DOM", async () => {
    apiKeyStatus.mockResolvedValue(false);
    renderPage();

    const input = (await screen.findByLabelText(/API key/)) as HTMLInputElement;
    expect(input.type).toBe("password");

    fireEvent.change(input, { target: { value: FAKE_KEY } });
    fireEvent.click(screen.getByRole("button", { name: /保存密钥/ }));

    await waitFor(() => expect(setApiKey).toHaveBeenCalledWith(FAKE_KEY));
    await waitFor(() => expect(input.value).toBe(""));
    expect(document.body.textContent ?? "").not.toContain(FAKE_KEY);
    expect(document.body.innerHTML).not.toContain(FAKE_KEY);
  });

  it("translates a keychain failure into Chinese copy instead of the raw code", async () => {
    setApiKey.mockRejectedValue("secret_error");
    renderPage();

    const input = (await screen.findByLabelText(/API key/)) as HTMLInputElement;
    fireEvent.change(input, { target: { value: FAKE_KEY } });
    fireEvent.click(screen.getByRole("button", { name: /保存密钥/ }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent ?? "").toContain("钥匙串");
    expect(alert.textContent ?? "").not.toContain("secret_error");
  });
});

describe("SettingsPage / base_url", () => {
  it("renders the engine's rejection as Chinese copy and no success notice", async () => {
    setBaseUrl.mockRejectedValue("invalid_url");
    renderPage();

    const input = (await screen.findByLabelText(/LLM 端点/)) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "file:///etc/passwd" } });
    fireEvent.click(screen.getByRole("button", { name: /保存端点/ }));

    const alert = await screen.findByRole("alert");
    expect(alert.textContent ?? "").toContain("http://");
    expect(alert.textContent ?? "").not.toContain("invalid_url");
    expect(screen.queryByText(/已保存/)).toBeNull();
  });

  it("saves an http(s) endpoint and reports success", async () => {
    renderPage();

    const input = (await screen.findByLabelText(/LLM 端点/)) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "https://api.example.com/v1" } });
    fireEvent.click(screen.getByRole("button", { name: /保存端点/ }));

    await waitFor(() =>
      expect(setBaseUrl).toHaveBeenCalledWith("https://api.example.com/v1"),
    );
    expect(await screen.findByText(/已保存/)).toBeTruthy();
  });

  // D-16a / D-06：LLM 配置可跳过。页面在**没有任何密钥**时仍要完整渲染，
  // 不弹不可关闭的引导、不把用户挡在别处。
  it("stays fully usable with no key configured", async () => {
    apiKeyStatus.mockResolvedValue(false);
    renderPage();

    expect(await screen.findByText(/未配置/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /保存密钥/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /保存端点/ })).toBeTruthy();
    expect(screen.getByText(/不配置.*也可以/)).toBeTruthy();
  });
});
