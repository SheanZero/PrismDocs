import { useState } from "react";
import { devPing } from "./lib/ipc";

type PingState =
  | { status: "idle" }
  | { status: "pending" }
  | { status: "ok"; version: string }
  | { status: "error"; message: string };

export default function App() {
  const [ping, setPing] = useState<PingState>({ status: "idle" });

  async function handlePing() {
    setPing({ status: "pending" });
    try {
      const version = await devPing();
      setPing({ status: "ok", version });
    } catch (err) {
      setPing({ status: "error", message: String(err) });
    }
  }

  return (
    <main style={{ fontFamily: "system-ui, sans-serif", padding: "2rem", lineHeight: 1.6 }}>
      <h1>PrismDocs</h1>
      <p>
        Tracer 页：验证 React → Tauri command → prism-engine → prism-store → SQLite 全链路。
      </p>
      <button type="button" onClick={handlePing} disabled={ping.status === "pending"}>
        ping engine
      </button>
      <section aria-live="polite" style={{ marginTop: "1rem" }}>
        {ping.status === "idle" && <span>尚未调用。</span>}
        {ping.status === "pending" && <span>调用中…</span>}
        {ping.status === "ok" && (
          <span>
            SQLite version: <strong data-testid="sqlite-version">{ping.version}</strong>
          </span>
        )}
        {ping.status === "error" && (
          <span role="alert" style={{ color: "#b00020" }}>
            调用失败：{ping.message}
          </span>
        )}
      </section>
    </main>
  );
}
