import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

// 本文件测的是**可达性**，不是两个页面各自的行为——那两块各有自己的测试文件。
// 把页面换成哑桩，路由这条线才不会被 IPC / listen 的失败噪声盖住。
vi.mock("./pages/Settings", () => ({
  default: () => <div data-testid="settings-page">settings</div>,
}));
vi.mock("./pages/DevSmoke", () => ({
  default: () => <div data-testid="smoke-page">smoke</div>,
}));
vi.mock("./lib/useEngineInvalidation", () => ({
  useEngineInvalidation: () => {},
}));

import App from "./App";

// vitest 的 globals 未开启，Testing Library 的自动清理不会注册。
afterEach(() => {
  cleanup();
  vi.unstubAllEnvs();
});

beforeEach(() => {
  // hash 是 jsdom 里跨用例残留的全局状态：上一条用例点进 #/dev 之后不复位，
  // 下一条以为自己在默认路由上，实际是从冒烟页开始的。
  window.location.hash = "";
});

describe("App routing", () => {
  // 机制回归：hash 路由本身不能被这次修复换掉。
  it("renders the smoke page when the hash is already #/dev", async () => {
    window.location.hash = "#/dev";
    render(<App />);
    expect(await screen.findByTestId("smoke-page")).toBeTruthy();
  });

  it("renders the settings page on the default route", async () => {
    render(<App />);
    expect(await screen.findByTestId("settings-page")).toBeTruthy();
    expect(screen.queryByTestId("smoke-page")).toBeNull();
  });
});

// 这一组测的是「用户到得了吗」，与「hash 等于 #/dev 时渲染谁」是两件事。
// 后者在冒烟页完全够不着的情况下也是绿的——本阶段真实发生过：单测全绿，
// 而 Tauri 窗口没有地址栏，用户根本无法输入 hash。
describe("App / dev smoke page reachability", () => {
  it("reaches the smoke page from the default route by clicking, with no address bar", async () => {
    render(<App />);
    await screen.findByTestId("settings-page");

    const entry = screen.getByTestId("dev-route-entry");
    // 必须是可点的控件，不能是一行「请手动输入 #/dev」的说明文字。
    expect(entry.tagName).toBe("BUTTON");

    fireEvent.click(entry);

    expect(await screen.findByTestId("smoke-page")).toBeTruthy();
    // 点击要真的改到地址上，而不是只在组件内部记了个 state——
    // 否则刷新即失效，且与 hash 机制脱节。
    await waitFor(() => expect(window.location.hash).toBe("#/dev"));
  });

  it("returns from the smoke page to settings by clicking", async () => {
    window.location.hash = "#/dev";
    render(<App />);
    await screen.findByTestId("smoke-page");

    fireEvent.click(screen.getByTestId("dev-route-back"));

    expect(await screen.findByTestId("settings-page")).toBeTruthy();
    expect(screen.queryByTestId("smoke-page")).toBeNull();
  });

  // D-06 承诺的是「不放导航入口」的正式产品外观；正式构建里这个入口必须整个不存在，
  // 而不是靠"没人会发现"。import.meta.env.DEV 在 vite build 下是字面 false，会被摇掉。
  it("has no dev entry at all in a production build", async () => {
    vi.stubEnv("DEV", false);
    render(<App />);

    await screen.findByTestId("settings-page");
    expect(screen.queryByTestId("dev-route-entry")).toBeNull();
    expect(screen.queryByTestId("smoke-page")).toBeNull();
  });

  it("also hides the back control in a production build", async () => {
    vi.stubEnv("DEV", false);
    window.location.hash = "#/dev";
    render(<App />);

    await screen.findByTestId("smoke-page");
    expect(screen.queryByTestId("dev-route-back")).toBeNull();
  });
});
