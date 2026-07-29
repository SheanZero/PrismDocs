import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";

import { errorCopy, EVENT_CHANGED, LISTEN_FAILED, type EngineEvent } from "./ipc";

/// notify-then-fetch 的前端半边（D-07 的最终模式）：引擎的粗粒度事件进来，
/// TanStack Query 失效并重取，SQLite 始终是唯一真相源——事件本身不携带数据。
///
/// **effect 的返回值必须是清理函数。** `listen` 返回的是 `Promise<UnlistenFn>`，
/// 所以清理写成 `() => { pending.then((un) => un()); }`。漏掉它是这个 pattern 最常见的
/// bug：React 19 StrictMode 下 effect 执行两次，于是注册两个 listener，一条事件触发
/// N 次 refetch，且每次热更新翻倍。它不报错，只是变慢——所以由单测盯着（Pitfall 5）。
///
/// **返回值是失败文案（成功时为 null），调用方必须把它渲染出来。** `listen` 走插件命令
/// `plugin:event|listen`，受 Tauri ACL 管辖；capability 缺失时它 reject，而同一页面上
/// `invoke` 的自有命令不过 ACL、照常成功。没有 `.catch` 的话整条失效链路就此静默失能：
/// 不报错、不刷新、看起来只是「引擎没发事件」。对一个以「0 静默丢失」为发布门槛的项目，
/// 这一类失败不能被丢进未处理的 Promise。
export function useEngineInvalidation(): string | null {
  const queryClient = useQueryClient();
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const pending = listen<EngineEvent>(EVENT_CHANGED, (event) => {
      const payload = event.payload;
      if (payload.kind === "resync") {
        // 全量失效。Lagged 之后丢了多少条、丢的是哪些 project 都不可知，
        // 按 key 失效等于替不可知的事实做假设。
        //
        // `void` 是 01-26 的 lint 闸门首跑加上的：`invalidateQueries` 返回 Promise，
        // 而这里刻意不等它（失效是 fire-and-forget，等它等于把事件处理器阻塞到 refetch 落定）。
        // 与 `Settings.tsx` / `DevSmoke.tsx` 已有的四个调用点同形——那四处本来就写了 `void`，
        // 本文件这两处漏了，闸门装上的第一跑就把这条不一致点了出来。
        void queryClient.invalidateQueries();
        return;
      }
      void queryClient.invalidateQueries({ queryKey: ["docs", payload.projectId] });
    });

    pending.catch(() => {
      if (active) setFailure(errorCopy(LISTEN_FAILED));
    });

    return () => {
      active = false;
      // 两参形式而不是 .then(un => un())：后者在 listen 失败时自己也是一个未处理的 rejection。
      pending.then(
        (unlisten) => unlisten(),
        () => {},
      );
    };
  }, [queryClient]);

  return failure;
}
