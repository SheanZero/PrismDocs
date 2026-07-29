//! 总线 → Tauri event 适配（Pattern 5，notify-then-fetch）。
//!
//! Tauri 官方对事件系统的两条警告是本模块存在的理由：事件监听器在发射端快速连发时
//! **可能乱序处理**，且事件载荷永远是 JSON 字符串、缺少命令具备的 capability 控制。
//! 因此这条通路上跑的只能是**粗粒度失效信号**（`EngineEvent` 只带 ID 与计数），
//! 前端收到后自己回查 SQLite。有序、请求作用域的数据投递走 `tauri::ipc::Channel`
//! （见 [`crate::smoke`]）。
//!
//! 本文件与 [`crate::commands`] 是 workspace 中仅有的两处 link tauri 的地方；
//! engine 侧对 tauri 一无所知（D-01）。

use prism_types::EngineEvent;
use tauri::{AppHandle, Emitter};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;

/// 前端 `listen()` 的事件名。改动即前后端契约变更。
pub const EVENT_CHANGED: &str = "prism://changed";

/// 「收到什么 → 该做什么」。
///
/// 单独抽成枚举 + 纯函数（而不是直接写在 `spawn` 的 `match` 里）只有一个目的：
/// 让这三条映射**脱离 tauri 运行时**可被单测。`Lagged` 那一条的失败模式是
/// 前端静默停留在旧数据上、不报任何错——只有测试能守住它。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusOutcome {
    /// 原样转发一条失效信号。
    Emit(EngineEvent),
    /// 补一次全量失效。
    Resync,
    /// 发送端已全部销毁，退出循环。
    Stop,
}

/// 把 `broadcast::Receiver::recv()` 的结果映射为壳层动作。
///
/// `Lagged` 分支不是可选的：`tokio::sync::broadcast` 用有界环形缓冲，接收端落后时
/// 会**丢弃最旧消息**并返回 `RecvError::Lagged(n)`。因为载荷是粗粒度失效信号，
/// 丢消息可以用一次 `Resync`（全量失效）无损补偿——这正是 notify-then-fetch
/// 相对 push-payload 的全部价值所在。漏掉它不会报错，只会让前端一直看着旧数据。
pub fn map_recv(received: Result<EngineEvent, RecvError>) -> BusOutcome {
    match received {
        Ok(ev) => BusOutcome::Emit(ev),
        Err(RecvError::Lagged(_)) => BusOutcome::Resync,
        Err(RecvError::Closed) => BusOutcome::Stop,
    }
}

/// 起一个常驻任务，把引擎总线的每条事件转发成一次 `prism://changed` 发射。
///
/// 发射失败只记 `warn` 不中断循环：窗口已关闭时发射失败是正常情况，
/// 为此杀掉适配器会让后续新开的窗口收不到任何事件。
pub fn spawn(app: AppHandle, mut rx: Receiver<EngineEvent>) {
    tauri::async_runtime::spawn(async move {
        loop {
            match map_recv(rx.recv().await) {
                BusOutcome::Emit(ev) => emit(&app, ev),
                BusOutcome::Resync => emit(&app, EngineEvent::Resync),
                BusOutcome::Stop => break,
            }
        }
    });
}

fn emit(app: &AppHandle, ev: EngineEvent) {
    if let Err(err) = app.emit(EVENT_CHANGED, ev) {
        tracing::warn!(error = %err, "failed to emit an invalidation event");
    }
}

#[cfg(test)]
mod tests {
    use super::{map_recv, BusOutcome, EVENT_CHANGED};
    use prism_types::EngineEvent;
    use tokio::sync::broadcast::error::RecvError;

    fn doc_changed() -> EngineEvent {
        EngineEvent::DocChanged {
            project_id: "p1".into(),
            doc_id: "d1".into(),
        }
    }

    #[test]
    fn bus_adapter_maps_event_to_emit() {
        assert_eq!(map_recv(Ok(doc_changed())), BusOutcome::Emit(doc_changed()));
    }

    #[test]
    fn lagged_maps_to_resync() {
        assert_eq!(map_recv(Err(RecvError::Lagged(7))), BusOutcome::Resync);
    }

    #[test]
    fn closed_stops_loop() {
        assert_eq!(map_recv(Err(RecvError::Closed)), BusOutcome::Stop);
    }

    #[test]
    fn bus_adapter_event_name_is_the_frontend_contract() {
        assert_eq!(EVENT_CHANGED, "prism://changed");
    }
}
