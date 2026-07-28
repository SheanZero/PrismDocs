import { invoke } from "@tauri-apps/api/core";

/// 前端唯一的 invoke 封装点——后续 plan 在此追加命令，不在组件里直接调 invoke。

export async function devPing(): Promise<string> {
  return await invoke<string>("dev_ping");
}
