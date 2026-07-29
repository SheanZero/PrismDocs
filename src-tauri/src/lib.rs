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

pub fn run() {
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
        .invoke_handler(tauri::generate_handler![commands::dev_ping])
        .run(tauri::generate_context!())
        .expect("failed to start the PrismDocs shell");
}
