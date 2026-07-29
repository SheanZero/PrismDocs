//! 总线 → Tauri event 适配（Pattern 5，notify-then-fetch）。
//!
//! RED 阶段：本文件当前只有测试，`map_recv` / `BusOutcome` / `EVENT_CHANGED`
//! 尚不存在，编译即失败。GREEN 由同一 task 的下一个 commit 落地。

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
