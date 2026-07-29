import { useEffect, useState } from "react";

import { useEngineInvalidation } from "./lib/useEngineInvalidation";
import DevSmokePage from "./pages/DevSmoke";
import SettingsPage from "./pages/Settings";

const DEV_ROUTE = "#/dev";
const HOME_ROUTE = "#/";

function useHashRoute(): string {
  const [hash, setHash] = useState(() => window.location.hash);
  useEffect(() => {
    const onChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onChange);
    return () => window.removeEventListener("hashchange", onChange);
  }, []);
  return hash;
}

/// dev-only 的路由开关。**Tauri 窗口没有地址栏**——原先「靠地址栏进入 #/dev」的设想
/// 在真实 runtime 里不可执行，用户到不了冒烟页，而单测因为只断言「hash 是 #/dev 时渲染谁」
/// 照样全绿。这里补的是可达性，不是导航系统。
///
/// D-06 说的「隐藏、不放导航入口」是**正式产品外观**的承诺，不是「不可达」。
/// `import.meta.env.DEV` 在 vite build 下被替换成字面 false，整块会被摇掉：
/// 正式构建里这个按钮不存在，而不是藏起来。
function DevRouteToggle({ onDevRoute }: { onDevRoute: boolean }) {
  if (!import.meta.env.DEV) return null;

  return (
    <button
      type="button"
      data-testid={onDevRoute ? "dev-route-back" : "dev-route-entry"}
      // 改 location.hash 而不是只改组件 state：走的仍是原来那条 hashchange 通路，
      // 刷新后停留在同一页，也没有第二套路由机制。
      onClick={() => {
        window.location.hash = onDevRoute ? HOME_ROUTE : DEV_ROUTE;
      }}
      style={toggle}
    >
      {onDevRoute ? "← 设置" : "dev 冒烟页"}
    </button>
  );
}

/// Phase 1 的壳只有两块（D-06）：设置页与隐藏冒烟页。
/// **不建正式布局、不建文档树、不建侧栏**——Phase 2 才有导入功能，现在排布局是推测式设计。
///
/// 路由用最简单的 hash 比较；dev 构建下额外挂一个角落里的开关供人工验证使用。
export default function App() {
  const route = useHashRoute();
  const onDevRoute = route === DEV_ROUTE;

  // 失效链路在顶层挂**一次**：事件是粗粒度的，每个页面各挂一次只会重复失效。
  useEngineInvalidation();

  return (
    <>
      {onDevRoute ? <DevSmokePage /> : <SettingsPage />}
      <DevRouteToggle onDevRoute={onDevRoute} />
    </>
  );
}

const toggle = {
  position: "fixed",
  right: "0.75rem",
  bottom: "0.75rem",
  fontSize: "0.75rem",
  padding: "0.25rem 0.6rem",
  opacity: 0.6,
} as const;
