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

/// `settings` 表里 LLM 端点的键名（Rust 侧 `prism_store::settings::SETTING_BASE_URL`）。
export const SETTING_BASE_URL = "llm.base_url";

// ---------------------------------------------------------------- 错误码 → 文案
//
// 命令的错误是**稳定短码串**，不是给人读的句子。短码是给代码分支用的契约；
// 面向用户的文案在这里生成，且**从不把码串本身放进 UI**——那既没用也是内部细节。

const ERROR_COPY: Record<string, string> = {
  invalid_url: "链接必须以 http:// 或 https:// 开头，并带有主机名。",
  invalid_setting: "这个配置项不被接受（疑似密钥的键名一律不入库）。",
  store_error: "写入本地数据库失败，请重试。",
  secret_error: "系统钥匙串当前不可用，密钥没有保存。",
  task_failed: "后台任务没能完成，请重试。",
  channel_send_failed: "数据流通道已关闭。",
  engine_error: "引擎遇到一个内部错误。",
  listen_failed: "事件通道未能建立，界面不会随引擎变更自动刷新。",
};

/// `listen()` 建不起来时用的码。
///
/// 它不是引擎命令返回的短码，而是**前端自造**的一个：`listen` 走的是插件命令
/// `plugin:event|listen`，被 Tauri ACL 拒绝时 reject 出来的是一段原始英文 ACL 文本
/// （"event.listen not allowed. Permissions associated with this command: …"）。
/// 那段文本既不该进 DOM，也不该走 `errorCopy` 的兜底——兜底文案「操作失败，请重试」
/// 会把「本页从此收不到任何事件」说成一次可重试的操作失败。
export const LISTEN_FAILED = "listen_failed";

/// 把命令错误译成中文文案。
///
/// 无法识别时给的是**通用**兜底，而不是 `String(err)`：把未知内容原样渲染进 DOM
/// 正是「内部细节泄漏到界面」的常见入口，而错误对象里可能恰好带着不该露面的东西。
export function errorCopy(err: unknown): string {
  const code = typeof err === "string" ? err : "";
  return ERROR_COPY[code] ?? "操作失败，请重试。";
}

// ---------------------------------------------------------------- 命令封装
//
// 每个函数一行 invoke。

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

/// 删除已保存的密钥。幂等：已不存在视为已删除。
export async function deleteApiKey(): Promise<void> {
  await invoke("delete_api_key");
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

/// 写入样例文档，返回它们所属的 project id（由引擎给出，前端不另抄常量）。
export async function devSeedSampleDocs(): Promise<string> {
  return await invoke<string>("dev_seed_sample_docs");
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
