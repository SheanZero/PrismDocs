//! PrismDocs 桌面壳——本 workspace 中唯一 link tauri 的 crate（D-01）。
//!
//! 壳只做三件事：解析 sidecar 路径、装配 facade、把命令注册进 IPC。
//! 任何业务逻辑都属于 engine 侧（Anti-Pattern 1）。

pub mod bus_adapter;
pub mod commands;
pub mod smoke;

use std::sync::Arc;

use prism_engine::Engine;
use prism_store::{default_db_path, Store};

pub struct AppState {
    pub engine: Arc<Engine>,
}

impl AppState {
    /// 装配 facade：sidecar 路径 → store → engine。
    pub fn bootstrap() -> Result<AppState, Box<dyn std::error::Error>> {
        let db_path = default_db_path()?;
        let store = Store::open(&db_path)?;
        Ok(AppState {
            engine: Arc::new(Engine::new(Arc::new(store))),
        })
    }
}

/// 默认过滤档位。三条安全决策的日志（`middleware.rs` 的无差别 403 真实原因、
/// `settings.rs` 的明文 http 告警、下面那条钥匙串不可用降级提示）都是 `warn!`，
/// `info` 已经全部覆盖；刻意**不**给 prism_mcp 单开 `debug`——少开一个 target 的 debug，
/// 就少一份「日志里到底会出现什么」的不确定性（T-01-58：这个 sink 本身是新增的外泄面）。
const DEFAULT_LOG_FILTER: &str = "info";

/// 安装全局 tracing subscriber。返回 `true` 表示**本次调用**完成了安装。
///
/// 收尾用 `try_init()` 而不是 `init()`：后者在全局 dispatcher 已就位时会 panic，
/// 而「装日志这件事本身把应用弄崩」是最不该发生的失败模式。返回 bool 让调用方与测试
/// 都能分辨「这次装上了」与「早就装好了」。
///
/// 过滤档位优先读 `RUST_LOG`；读不到或解析失败时回落到 [`DEFAULT_LOG_FILTER`]。
pub fn init_tracing() -> bool {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(DEFAULT_LOG_FILTER));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .is_ok()
}

pub fn run() {
    // 必须是第一步：装在 `tauri::Builder` 之后就漏掉了 `AppState::bootstrap()` 的失败
    // 与下面「钥匙串不可用」那条降级提示。返回值显式丢弃——装不上不该阻断启动，
    // 与「钥匙串后端注册失败不阻断启动」同一口径。
    let _ = init_tracing();

    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;

            let state = AppState::bootstrap()?;

            // 钥匙串后端注册失败**不阻断启动**（D-06：无 key 时应用照常启动）。
            // 把它冒泡给 setup 会让「登录钥匙串被锁」变成开不了窗口。
            #[cfg(target_os = "macos")]
            if let Err(err) = state.engine.init_secrets() {
                tracing::warn!(error = %err, "keychain backend unavailable; secrets are disabled");
            }

            bus_adapter::spawn(app.handle().clone(), state.engine.subscribe());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dev_ping,
            commands::search_documents,
            commands::set_api_key,
            commands::api_key_status,
            commands::delete_api_key,
            commands::get_setting,
            commands::set_base_url,
            commands::dev_emit_bus_event,
            commands::dev_smoke_stream,
            commands::dev_seed_sample_docs,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the PrismDocs shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 装上 subscriber 这件事本身没有任何行为面的观察点——workspace 里 7 处 `tracing`
    /// 发射点在没有 subscriber 时**照常编译、照常执行、照常返回**，只是写向虚无。
    /// 于是判别性只能落在 ②：全局 dispatcher 是否真的就位。
    ///
    /// 刻意**不写**「调用前 `has_been_set()` 为 false」这条前置断言——全局 dispatcher 是
    /// 进程级的，把前置条件写进判别性测试会让反证落在前置条件上而不是被守的那条断言上
    /// （本 phase 已记录的教训）。
    #[test]
    fn tracing_init_installs_a_global_subscriber_and_is_idempotent() {
        // ① 本次调用完成了安装
        assert!(init_tracing(), "the first init_tracing() should install");

        // ② 判别性所在：若 init_tracing 被写成什么都不做的桩，① 仍可能绿而这条一定红
        assert!(
            tracing::dispatcher::has_been_set(),
            "no global dispatcher after init_tracing() — the 7 tracing call sites \
             across the workspace are still writing into a null sink"
        );

        // ③ 重复调用安全：用 try_init 而非会 panic 的 init
        assert!(
            !init_tracing(),
            "the second init_tracing() should report that it did not install"
        );
    }

    /// `init_tracing()` 必须排在 `tauri::Builder` 之前。
    ///
    /// 这条断言看的是源码顺序，不是行为——`run()` 会阻塞直到窗口关闭，测不了。
    /// 顺序本身重要：装在 `Builder` 之后就漏掉了 `AppState::bootstrap()` 的失败与
    /// 「钥匙串不可用」那条降级提示，而那两条正是 WR-04 点名「无处可去」的日志。
    ///
    /// 定位刻意从 `pub fn run()` 起切片而不是全文 `find`：函数签名
    /// `pub fn init_tracing() -> bool` 本身就含 `init_tracing()` 这个子串，
    /// 全文 find 会永远命中定义处（它在 `run()` 之前），使断言恒真。
    ///
    /// 两个锚点都取**完整语句**而不是裸名字。执行本 plan 时实测过：锚点写成
    /// `"tauri::Builder"` 会命中 `run()` 里那条**解释性注释**中的 `tauri::Builder`
    /// 字样——它排在真正的调用之前，于是断言在实现完全正确时变红。源码序断言的
    /// 匹配面里同时含代码与注释，锚点必须窄到只有语句本身能命中。
    #[test]
    fn run_installs_tracing_before_it_builds_the_app() {
        let source = include_str!("lib.rs");
        let run_at = source
            .find("pub fn run()")
            .expect("lib.rs should declare pub fn run()");
        let body = &source[run_at..];

        let init_at = body
            .find("let _ = init_tracing();")
            .expect("run() should call init_tracing()");
        let builder_at = body
            .find("tauri::Builder::default()")
            .expect("run() should build the tauri app");

        assert!(
            init_at < builder_at,
            "init_tracing() runs after tauri::Builder — AppState::bootstrap() failures and \
             the keychain-unavailable degradation notice would both be lost"
        );
    }
}
