//! IPC 命令面。
//!
//! # 两条纪律
//!
//! ## 1. 每个命令都是对 facade 的**单行委托**（Anti-Pattern 1）
//!
//! 业务逻辑一旦写进命令体，它就离开了可脱离 tauri 单测的 engine 侧，
//! 也绕过了单写者与校验纪律。所有命令统一经 [`delegate`] 进入 `Engine`，
//! 于是「在命令里自己开一个连接」这件事在本文件的写法下无从表达。
//!
//! ## 2. 内部错误不原文透传前端（T-01-11）
//!
//! `EngineError` 的 `Display` 会带上 SQLite 的错误原文与钥匙串的失败描述。
//! 那些是内部事实，不是给用户看的。全部经 [`map_err`] 收敛成**稳定的短错误码串**——
//! 前端据串分支，跨版本不会因为下层措辞变化而失效。

use prism_engine::{Engine, EngineError};
use prism_store::StoreError;
use prism_types::{EngineEvent, SearchHit};
use tauri::State;

use crate::smoke::{self, SmokeEvent};
use crate::AppState;

/// `spawn_blocking` 的 join 失败（任务 panic 或运行时关闭）。
const ERR_TASK: &str = "task_failed";
/// Channel 已关闭 / 发送失败。
const ERR_CHANNEL: &str = "channel_send_failed";

/// 把内部错误收敛成稳定的短错误码串。
///
/// **绝不 `EngineError::to_string()`**：那会把内部细节（SQLite 语句片段、
/// 钥匙串平台错误码）送过 IPC 边界。`EngineError` 与 `StoreError` 都是
/// `#[non_exhaustive]`，兜底臂保证新变体默认落到最粗的类别，而不是编译失败后
/// 被人顺手加成 `to_string()`。
pub fn map_err(err: EngineError) -> String {
    match err {
        EngineError::Store(StoreError::InvalidUrl(_)) => "invalid_url",
        EngineError::Store(StoreError::InvalidSetting(_)) => "invalid_setting",
        EngineError::Store(_) => "store_error",
        EngineError::Llm(_) => "secret_error",
        _ => "engine_error",
    }
    .to_string()
}

/// 在阻塞线程池上跑一次 facade 调用。
///
/// facade 的方法是**同步**的（`std::sync::MutexGuard` 的 `!Send` 靠这一点在编译期
/// 挡住「跨 await 持写锁」），所以从 async 命令进入时必须 `spawn_blocking`，
/// 否则一次慢查询会卡住整个 IPC 线程。
async fn delegate<T, F>(state: &State<'_, AppState>, call: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&Engine) -> Result<T, EngineError> + Send + 'static,
{
    let engine = state.engine.clone();
    tauri::async_runtime::spawn_blocking(move || call(&engine))
        .await
        .map_err(|_| ERR_TASK.to_string())?
        .map_err(map_err)
}

#[tauri::command]
pub async fn dev_ping(state: State<'_, AppState>) -> Result<String, String> {
    delegate(&state, |engine| engine.ping()).await
}

#[tauri::command]
pub async fn search_documents(
    state: State<'_, AppState>,
    project_id: String,
    q: String,
) -> Result<Vec<SearchHit>, String> {
    delegate(&state, move |engine| engine.search(&project_id, &q)).await
}

/// 写入 LLM API key。密钥只经这一个方向单向进入，**没有任何命令把它读回来**。
#[tauri::command]
pub async fn set_api_key(state: State<'_, AppState>, secret: String) -> Result<(), String> {
    delegate(&state, move |engine| engine.set_api_key(&secret)).await
}

/// 是否已配置 API key。**返回布尔，不返回密钥**（T-01-04b）。
#[tauri::command]
pub async fn api_key_status(state: State<'_, AppState>) -> Result<bool, String> {
    delegate(&state, |engine| engine.api_key_status()).await
}

/// 删除 API key。幂等：已不存在视为已删除。
#[tauri::command]
pub async fn delete_api_key(state: State<'_, AppState>) -> Result<(), String> {
    delegate(&state, |engine| engine.delete_api_key()).await
}

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    delegate(&state, move |engine| engine.get_setting(&key)).await
}

#[tauri::command]
pub async fn set_base_url(state: State<'_, AppState>, url: String) -> Result<(), String> {
    delegate(&state, move |engine| engine.set_base_url(&url)).await
}

/// 冒烟页触发一次总线事件，用于验证 notify-then-fetch 往返。
#[tauri::command]
pub async fn dev_emit_bus_event(
    state: State<'_, AppState>,
    project_id: String,
    doc_id: String,
) -> Result<(), String> {
    delegate(&state, move |engine| {
        engine.publish(EngineEvent::DocChanged { project_id, doc_id });
        Ok(())
    })
    .await
}

/// 冒烟页的样例数据入口：写入样例文档并返回它们所属的 project id。
#[tauri::command]
pub async fn dev_seed_sample_docs(state: State<'_, AppState>) -> Result<String, String> {
    delegate(&state, |engine| engine.seed_sample_docs()).await
}

/// Channel 有序流（Pattern 6）。
///
/// Channel 由**前端**创建并作为命令参数传入，因此这条通路只适合**请求作用域**的流；
/// 引擎主动推送没有常驻 channel，那正是 FS 驱动流程必须走事件通路
/// （[`crate::bus_adapter`]）的原因。
///
/// `total` 是不可信输入（WebView 里的任意脚本都够得着这条命令），上界由
/// [`smoke::SMOKE_MAX_TOTAL`] 在 [`smoke::generate`] 内部夹紧——T-01G-27。
/// 循环本身经 `spawn_blocking` 落到阻塞线程池：`Channel::send` 是**同步**的，
/// 这个循环一次都不让出，留在 async 上下文里就是本模块开头第 47-51 行说的
/// 「卡住整个 IPC 线程」——T-01G-28。
#[tauri::command]
pub async fn dev_smoke_stream(
    on_event: tauri::ipc::Channel<SmokeEvent>,
    total: u32,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || smoke::generate(total, |ev| on_event.send(ev)))
        .await
        .map_err(|_| ERR_TASK.to_string())?
        .map_err(|_| ERR_CHANNEL.to_string())
}

#[cfg(test)]
mod tests {
    /// 本文件的**生产代码**，去掉注释行与测试模块自身。
    ///
    /// 两处都必须去掉，否则下面的哨兵会被文档注释里的举例和断言自己的字符串
    /// 字面量命中而恒红（且看不出是被什么命中的）。形态沿用 01-07 的 facade 哨兵。
    fn production_source() -> String {
        let source = include_str!("commands.rs");
        let cut = source.find("#[cfg(test)]").unwrap_or(source.len());
        source[..cut]
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 命令体里不得出现 SQLite / 钥匙串的实现细节（T-01-14a）。
    ///
    /// 源码断言而非行为断言：一个「在命令里自己开连接」的实现不会有哪次调用失败，
    /// 能观测到的只有它写在哪一层。
    #[test]
    fn commands_carry_no_business_logic() {
        let source = production_source();
        for leak in ["Connection", "prepare", "query_row", "keyring"] {
            assert!(
                !source.contains(leak),
                "命令层出现了 {leak:?} —— 业务逻辑正在逃出可测试的 engine"
            );
        }
    }

    /// `dev_smoke_stream` 必须把整个循环交给阻塞线程池（T-01G-28）。
    ///
    /// 源码断言而非行为断言：`Channel::send` 是同步的，一个直接在 async 命令体里
    /// 跑完 `0..total` 的实现不会有哪次调用失败——能观测到的只有它把循环放在哪一层。
    /// （行为侧的 `Handle::try_current()` 探针取决于 `tauri::async_runtime` 当前
    /// 选的后端，那是实现细节，不适合当契约。）
    #[test]
    fn dev_smoke_stream_hands_the_loop_to_the_blocking_pool() {
        let source = production_source();
        let body = source
            .split_once("pub async fn dev_smoke_stream")
            .expect("找不到 dev_smoke_stream")
            .1
            // 后面若再有别的 item，切在它的属性行上，别把邻居的实现算进来。
            .split("\n#[")
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            body.contains("tauri::async_runtime::spawn_blocking(move || smoke::generate("),
            "dev_smoke_stream 没有把 smoke::generate 交给 spawn_blocking —— \
             这个循环会跑在 IPC executor 上: {body}"
        );
    }

    /// 错误串必须经 `map_err` 收敛（T-01-11）。
    ///
    /// `e.to_string()` 是内部错误原文透传前端的典型写法；它一旦出现在本文件里，
    /// 数据库与钥匙串的失败描述就跨过了 IPC 边界。
    #[test]
    fn no_command_forwards_the_raw_engine_error_text() {
        let source = production_source();
        for leak in ["e.to_string()", "err.to_string()", "|e| e.to_string"] {
            assert!(
                !source.contains(leak),
                "命令层出现了 {leak:?} —— 内部错误原文正在透传前端"
            );
        }
    }

    /// 错误码串是给前端分支用的稳定契约，钉死它们的字面值。
    #[test]
    fn error_codes_are_stable_short_strings() {
        use super::map_err;
        use prism_engine::EngineError;
        use prism_store::StoreError;

        assert_eq!(
            map_err(EngineError::from(StoreError::InvalidUrl(
                "scheme must be one of [\"http\", \"https\"]".into()
            ))),
            "invalid_url"
        );
        assert_eq!(
            map_err(EngineError::from(StoreError::InvalidSetting(
                "looks like a secret".into()
            ))),
            "invalid_setting"
        );
        assert_eq!(
            map_err(EngineError::from(prism_engine::LlmError::Keychain(
                "Password data is not valid UTF-8".into()
            ))),
            "secret_error"
        );
    }

    /// 映射结果里不得残留下层错误文本（T-01-11 的行为侧）。
    ///
    /// 上一条钉的是「码串等于什么」，这一条钉的是「下层说了什么都传不出去」——
    /// 若哪天有人把某个变体改成 `format!("store_error: {e}")`，上一条也会红，
    /// 但这一条给出的是**为什么**红。
    #[test]
    fn mapped_errors_do_not_carry_lower_layer_text() {
        use super::map_err;
        use prism_engine::EngineError;
        use prism_store::StoreError;

        let inner = "/Users/someone/Library/Application Support/PrismDocs/prismdocs.db";
        let mapped = map_err(EngineError::from(StoreError::InvalidSetting(inner.into())));
        assert!(
            !mapped.contains(inner) && !mapped.contains('/'),
            "错误映射把下层文本带了出去: {mapped}"
        );
    }
}
