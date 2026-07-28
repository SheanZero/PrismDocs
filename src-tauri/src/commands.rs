//! IPC 命令面。每个命令都必须是对 facade 的**单行委托**（Anti-Pattern 1）。

use crate::AppState;

#[tauri::command]
pub async fn dev_ping(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.engine.ping().map_err(|e| e.to_string())
}
