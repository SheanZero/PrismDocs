//! PrismDocs 桌面壳——本 workspace 中唯一 link tauri 的 crate（D-01）。
//!
//! 壳只做三件事：解析 sidecar 路径、装配 facade、把命令注册进 IPC。
//! 任何业务逻辑都属于 engine 侧（Anti-Pattern 1）。

pub mod commands;

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
            app.manage(AppState::bootstrap()?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::dev_ping])
        .run(tauri::generate_context!())
        .expect("failed to start the PrismDocs shell");
}
