import { useEffect, useState } from "react";

import { useEngineInvalidation } from "./lib/useEngineInvalidation";
import DevSmokePage from "./pages/DevSmoke";
import SettingsPage from "./pages/Settings";

const DEV_ROUTE = "#/dev";

function useHashRoute(): string {
  const [hash, setHash] = useState(() => window.location.hash);
  useEffect(() => {
    const onChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return hash;
}

/// Phase 1 的壳只有两块（D-06）：设置页与隐藏冒烟页。
/// **不建正式布局、不建文档树、不建侧栏**——Phase 2 才有导入功能，现在排布局是推测式设计。
///
/// 路由用最简单的 hash 比较：冒烟页是"隐藏"的，不放导航入口，靠地址栏进入。
export default function App() {
  const route = useHashRoute();

  // 失效链路在顶层挂**一次**：事件是粗粒度的，每个页面各挂一次只会重复失效。
  useEngineInvalidation();

  return route === DEV_ROUTE ? <DevSmokePage /> : <SettingsPage />;
}
