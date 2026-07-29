import { QueryClient } from "@tanstack/react-query";

/// 前端数据层的单例（D-07）。
///
/// 两个默认值都在实现 notify-then-fetch：失效由**引擎事件**驱动，不由时间或窗口焦点驱动。
/// `staleTime: 0` 让失效后的重取立刻发生；`refetchOnWindowFocus: false` 关掉焦点轮询——
/// 桌面应用的窗口焦点切换频繁，靠它重取只会掩盖事件通路是否真的通了。
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 0,
      refetchOnWindowFocus: false,
    },
  },
});
