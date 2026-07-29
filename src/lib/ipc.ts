import { Channel, invoke } from "@tauri-apps/api/core";

/// 前端唯一的 invoke 封装点——后续 plan 在此追加命令，不在组件里直接调 invoke。

// ---------------------------------------------------------------- 契约类型
//
// 下面三个类型与 Rust 侧的 serde 形态一一对应，改一边必须改另一边：
//   EngineEvent  crates/prism-types/src/event.rs  tag="kind" + camelCase 变体/字段
//   SmokeEvent   src-tauri/src/smoke.rs           tag="event" + content="data"
//   SearchHit    crates/prism-types/src/dto.rs    camelCase 字段

/// 引擎发出的**粗粒度失效信号**。载荷只带 ID 与计数，绝不带文档正文。
export type EngineEvent =
  | { kind: "docChanged"; projectId: string; docId: string }
  | { kind: "inboxUpdated"; projectId: string; unread: number }
  | { kind: "resync" };

/// `prism://changed`：shell 的 bus adapter 把每条 EngineEvent 发成一次这个事件。
export const EVENT_CHANGED = "prism://changed";

/// Channel 有序流的事件。
export type SmokeEvent =
  | { event: "started"; data: { total: number } }
  | { event: "tick"; data: { seq: number } }
  | { event: "finished"; data: { total: number } };

export interface SearchHit {
  docId: string;
  title: string | null;
  relPath: string;
}

// ---------------------------------------------------------------- 命令封装
//
// 每个函数一行 invoke。命令的错误是**稳定短码串**（invalid_url / store_error /
// secret_error …），不是给人读的句子——调用方据码分支并自己出中文文案。

export async function devPing(): Promise<string> {
  return await invoke<string>("dev_ping");
}

export async function searchDocuments(
  projectId: string,
  q: string,
): Promise<SearchHit[]> {
  return await invoke<SearchHit[]>("search_documents", { projectId, q });
}

export async function setApiKey(secret: string): Promise<void> {
  await invoke("set_api_key", { secret });
}

/// **返回布尔**。没有任何命令把密钥读回前端（T-01-04b）。
export async function apiKeyStatus(): Promise<boolean> {
  return await invoke<boolean>("api_key_status");
}

export async function getSetting(key: string): Promise<string | null> {
  return await invoke<string | null>("get_setting", { key });
}

export async function setBaseUrl(url: string): Promise<void> {
  await invoke("set_base_url", { url });
}

export async function devEmitBusEvent(
  projectId: string,
  docId: string,
): Promise<void> {
  await invoke("dev_emit_bus_event", { projectId, docId });
}

/// Channel 由**前端**创建后作为命令参数传入——这条通路只适合请求作用域的流。
export async function devSmokeStream(
  total: number,
  onEvent: (ev: SmokeEvent) => void,
): Promise<void> {
  const channel = new Channel<SmokeEvent>();
  channel.onmessage = onEvent;
  await invoke("dev_smoke_stream", { onEvent: channel, total });
}
