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

  // 从供应商控制台复制密钥的正常结果就是带一个尾随换行或空格。判空用 `trim()`
  // 却把**原值**交出去，钥匙串里就躺着一个带空白的凭据：`api_key_status()` 报「已配置」，
  // 而 Phase 4 的每次调用返回 401，本地没有任何信号指向空白（与 01-16 的 `McpDeps::new` 同源）。
  it("trims surrounding whitespace off the key before it reaches the keychain", async () => {
    renderPage();

    const input = (await screen.findByLabelText(/API key/)) as HTMLInputElement;
    fireEvent.change(input, { target: { value: `  ${FAKE_KEY}\n` } });
    fireEvent.click(screen.getByRole("button", { name: /保存密钥/ }));

    // 逐字相等，不是 `toContain`：后者对一个原样透传的实现同样会绿。
    await waitFor(() => expect(setApiKey).toHaveBeenCalledWith(FAKE_KEY));
  });

  // 被拒的查询在 TanStack Query v5 里以 `isPending === false` + `data === undefined` 落定，
  // 于是「`data ? 已配置 : 未配置`」会**肯定地**声称没有配置密钥——而实际是钥匙串被锁、
  // 权限被拒或 IPC 失败。用户对「未配置」的反应是重新输入密钥，那次保存也会失败，
  // 且他仍不知道原因。
  it("says 读取失败 rather than 未配置 when the key status query is rejected", async () => {
    apiKeyStatus.mockRejectedValue(new Error("keychain is locked"));
    renderPage();

    expect(await screen.findByText(/读取失败/)).toBeTruthy();
    // 判别性所在：只断言「出现了读取失败」的话，一个同时还显示「未配置」的实现也会绿。
    expect(screen.queryByText(/未配置/)).toBeNull();

    const alert = await screen.findByRole("alert");
    // 文案等于 `errorCopy` 的产物，而不是 `String(error)`——错误对象里可能带着不该露面的东西。
    expect(alert.textContent).toBe("操作失败，请重试。");
    expect(alert.textContent ?? "").not.toContain("keychain is locked");
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

  // WINDOWS #10 / 01-22 遗留：成功通知此前不在任何 live region 里，读屏对「已保存」
  // 完全静默。冒烟页（01-22）已经是 error → alert / ok → status，这里补齐同一形状。
  //
  // 两条断言成对：只有前者时，一个把**所有**通知都改成 `alert` 的实现也会绿——
  // 而那会让每次成功保存都打断读屏用户正在读的内容。
  it("announces a successful save in a status region, not an alert region", async () => {
    renderPage();

    const field = (await screen.findByLabelText(/LLM 端点/)) as HTMLInputElement;
    fireEvent.change(field, { target: { value: "https://api.example.com/v1" } });
    fireEvent.click(screen.getByRole("button", { name: /保存端点/ }));

    const status = await screen.findByRole("status");
    expect(status.textContent ?? "").toContain("已保存");
    expect(screen.queryByRole("alert")).toBeNull();
  });

  // 凭据藏在**值**里：`llm.base_url` 这个键名再正常不过，而 `https://u:key@host/v1`
  // 里的那串东西会被原样提交。权威守卫在 engine 的 set_setting 写入路径上（01-10 Task 1），
  // 这一层只是让用户在按下保存之前就知道那串东西不该填在这里。
  //
  // 两条断言缺一不可（同 72-80 的范式）：只有 ① 时，一个**根本没提交表单**的组件也会通过；
  // 只有 ② 时，一个「按钮坏了」的组件也会通过。
  it("rejects a credential-bearing endpoint before it ever reaches the engine", async () => {
    const CREDENTIAL_URL = `https://u:${FAKE_KEY}@api.vendor.com/v1`;
    renderPage();

    const input = (await screen.findByLabelText(/LLM 端点/)) as HTMLInputElement;
    fireEvent.change(input, { target: { value: CREDENTIAL_URL } });
    fireEvent.click(screen.getByRole("button", { name: /保存端点/ }));

    // ① 本地就给出了专属文案（中文规则文案，不是原始码串）。
    const alert = await screen.findByRole("alert");
    expect(alert.textContent ?? "").toMatch(/用户名|密码/);
    expect(alert.textContent ?? "").not.toContain("invalid_url_credentials");

    // ② 一次 IPC 都没发出去——engine 侧的守卫不该是用户第一次听说这件事的地方。
    expect(setBaseUrl).not.toHaveBeenCalled();

    // ③ 文案不回显输入（T-01-26 同源）：被误填进端点栏的很可能就是密钥本身。
    expect(alert.textContent ?? "").not.toContain(FAKE_KEY);

    // ④ 阴性对照：守卫若写成「一律拒绝」，上面三条也都会绿——而那会把设置页彻底废掉。
    fireEvent.change(input, { target: { value: "https://api.example.com/v1" } });
    fireEvent.click(screen.getByRole("button", { name: /保存端点/ }));
    await waitFor(() =>
      expect(setBaseUrl).toHaveBeenCalledWith("https://api.example.com/v1"),
    );
    expect(setBaseUrl).toHaveBeenCalledTimes(1);
  });

  // 前端的端点判定面必须与 engine 侧 `validate_base_url` 逐项对齐。**两个方向**的分歧
  // 都要防：「前端放过、engine 拒绝」只是多一次 IPC 往返，而「前端拒绝、engine 会接受」
  // 会告诉用户一句自相矛盾的话——他输入 `HTTPS://…` 却被告知「链接必须以 https:// 开头」。
  //
  // `localUrlIssue` 是模块内私有函数，这里走 UI 行为（输入 → 保存 → 断言是否发出 IPC
  // 及其文案），那也更贴近真实路径。
  const URL_CASES: ReadonlyArray<{
    input: string;
    verdict: "accepted" | "invalid_url" | "invalid_url_credentials";
  }> = [
    { input: "https://api.example.com/v1", verdict: "accepted" },
    // scheme 大小写不敏感；engine 侧 `url` crate 会小写化后再比对 scheme。
    { input: "HTTPS://api.example.com/v1", verdict: "accepted" },
    { input: "HTTP://localhost:8080", verdict: "accepted" },
    { input: "ftp://api.example.com", verdict: "invalid_url" },
    { input: "not a url", verdict: "invalid_url" },
    { input: "https://", verdict: "invalid_url" },
    { input: "https://prism-test-user:prism-test-value@api.vendor.com/v1", verdict: "invalid_url_credentials" },
    // 只有用户名没有密码：`password` 在这里是空串，只看密码的守卫会漏掉它。
    { input: "https://prism-test-user@api.vendor.com/v1", verdict: "invalid_url_credentials" },
    { input: "https://api.vendor.com/v1?deployment=prism-test-value", verdict: "invalid_url_credentials" },
    { input: "https://api.vendor.com/v1#prism-test-value", verdict: "invalid_url_credentials" },
  ];

  it.each(URL_CASES)("judges $input as $verdict, matching the engine", async ({ input, verdict }) => {
    renderPage();

    const field = (await screen.findByLabelText(/LLM 端点/)) as HTMLInputElement;
    fireEvent.change(field, { target: { value: input } });
    fireEvent.click(screen.getByRole("button", { name: /保存端点/ }));

    if (verdict === "accepted") {
      await waitFor(() => expect(setBaseUrl).toHaveBeenCalledWith(input.trim()));
      expect(screen.queryByRole("alert")).toBeNull();
      return;
    }

    const alert = await screen.findByRole("alert");
    // 两条文案的判别词互不重叠——只断言「有 alert」的话，一个把所有拒绝都
    // 说成同一句的实现也会绿，而那正是 engine 侧特意分开两类的理由。
    if (verdict === "invalid_url") {
      expect(alert.textContent ?? "").toMatch(/并带有主机名/);
      expect(alert.textContent ?? "").not.toMatch(/用户名或密码/);
    } else {
      expect(alert.textContent ?? "").toMatch(/用户名或密码/);
    }
    // engine 侧的守卫不该是用户第一次听说这件事的地方。
    expect(setBaseUrl).not.toHaveBeenCalled();
  });

  it("does not claim 未设置 when the endpoint query is rejected", async () => {
    getSetting.mockRejectedValue(new Error("db is unreadable"));
    renderPage();

    expect(await screen.findByText(/读取失败/)).toBeTruthy();
    expect(screen.queryByText(/（未设置）/)).toBeNull();

    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toBe("操作失败，请重试。");
    expect(alert.textContent ?? "").not.toContain("db is unreadable");
  });

  // 阴性对照：真的没设置与读失败必须**可区分**。少了这一条，一个无条件显示
  // 「读取失败」的实现会让上面那条绿——而它把一个完全正常的空状态说成了故障。
  it("still says （未设置） when the endpoint query resolves to null", async () => {
    getSetting.mockResolvedValue(null);
    renderPage();

    expect(await screen.findByText(/（未设置）/)).toBeTruthy();
    expect(screen.queryByText(/读取失败/)).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
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
