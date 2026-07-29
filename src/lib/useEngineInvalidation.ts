import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";

import { EVENT_CHANGED, type EngineEvent } from "./ipc";

/// notify-then-fetch 的前端半边（D-07 的最终模式）：引擎的粗粒度事件进来，
/// TanStack Query 失效并重取，SQLite 始终是唯一真相源——事件本身不携带数据。
///
/// **effect 的返回值必须是清理函数。** `listen` 返回的是 `Promise<UnlistenFn>`，
/// 所以清理写成 `() => { pending.then((un) => un()); }`。漏掉它是这个 pattern 最常见的
/// bug：React 19 StrictMode 下 effect 执行两次，于是注册两个 listener，一条事件触发
/// N 次 refetch，且每次热更新翻倍。它不报错，只是变慢——所以由单测盯着（Pitfall 5）。
export function useEngineInvalidation() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const pending = listen<EngineEvent>(EVENT_CHANGED, (event) => {
      const payload = event.payload;
      if (payload.kind === "resync") {
        // 全量失效。Lagged 之后丢了多少条、丢的是哪些 project 都不可知，
        // 按 key 失效等于替不可知的事实做假设。
        queryClient.invalidateQueries();
        return;
      }
      queryClient.invalidateQueries({ queryKey: ["docs", payload.projectId] });
    });

    return () => {
      pending.then((unlisten) => unlisten());
    };
  }, [queryClient]);
}
