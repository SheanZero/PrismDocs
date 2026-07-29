//! 事件总线：engine 唯一的订阅点（RESEARCH § Architectural Responsibility Map）。
//!
//! 载荷是 [`EngineEvent`]——**粗粒度失效信号**，只带 ID 与计数，不带文档正文。
//! 这条设计前提决定了本文件的两个反直觉选择：
//!
//! 1. **无订阅者时 `publish` 不报错**（见下）。
//! 2. **慢订阅者丢消息是可接受的**：`tokio::sync::broadcast` 的有界环形缓冲在接收端
//!    落后时丢弃最旧消息并让下一次 `recv()` 返回 `RecvError::Lagged`。丢一条失效信号
//!    可以由一次 [`EngineEvent::Resync`] 无损补偿——shell adapter（plan 01-08）负责
//!    把 `Lagged` 翻译成 `Resync`。若走 push-payload，丢的就是不可恢复的业务数据了。
//!
//! 本文件不依赖 tauri（D-01）：总线是纯 tokio 的，shell 侧的 adapter 才认识 Tauri event。

use prism_types::EngineEvent;
use tokio::sync::broadcast;

/// broadcast 环形缓冲容量。
///
/// 256 的取法：足够吸收一次 FS 批量变更（Phase 2 的 watcher 一次 debounce 窗口内
/// 的文档变更量远小于此），又不至于让一个慢接收端把大量陈旧信号长期滞留在缓冲里。
/// 溢出不是故障——它退化成一次 `Lagged` → `Resync`。
pub const BUS_CAPACITY: usize = 256;

/// 类型化广播总线。每个订阅者独立收到**每一条**事件（广播，不是竞争消费）。
pub struct EventBus(broadcast::Sender<EngineEvent>);

impl EventBus {
    pub fn new() -> Self {
        // 丢弃建构时的那个 receiver：总线的生命周期由 Sender 持有。
        // 保留它会让 `subscriber_count()` 永远至少是 1，也会让缓冲从不被消费。
        let (tx, _rx) = broadcast::channel(BUS_CAPACITY);
        Self(tx)
    }

    /// 新增一个独立订阅者。
    ///
    /// 只能收到**订阅之后**发布的事件——broadcast 不重放历史。
    pub fn subscribe(&self) -> broadcast::Receiver<EngineEvent> {
        self.0.subscribe()
    }

    /// 广播一条失效信号。
    ///
    /// **返回单元值，内部吞掉 `SendError`。** `broadcast::Sender::send` 在一个接收端
    /// 都没有时返回 `Err(SendError(ev))`，但对粗粒度失效信号而言「没人在听」是**空输入
    /// 的正确语义**，不是错误：应用启动早期、shell adapter 尚未 spawn、或前端窗口已关闭时
    /// 都会出现。把它冒泡给调用方，等于让 Phase 2+ 的每一条写路径都长出一个
    /// 无意义的错误分支——而那个分支唯一合理的写法还是 `let _ = …`。
    ///
    /// 吞掉时记一行 `trace`：事件本身只有 ID 与计数，不含用户内容，可安全入日志。
    pub fn publish(&self, ev: EngineEvent) {
        if self.0.send(ev).is_err() {
            tracing::trace!("engine event published with no subscribers attached");
        }
    }

    /// 当前活跃订阅者数。测试的前置条件断言用；生产路径不该依赖它做分支。
    pub fn subscriber_count(&self) -> usize {
        self.0.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `publish` 的返回类型是本模块最容易被"顺手改回 Result"的地方——
    /// 一旦改了，无订阅者就重新变成调用方要处理的错误。这条断言让它编译不过。
    #[test]
    fn publish_returns_the_unit_value() {
        let bus = EventBus::new();
        let outcome: () = bus.publish(EngineEvent::Resync);
        assert_eq!(outcome, ());
    }

    #[test]
    fn a_fresh_bus_has_no_subscribers() {
        assert_eq!(EventBus::new().subscriber_count(), 0);
        assert_eq!(EventBus::default().subscriber_count(), 0);
    }

    #[test]
    fn subscribing_twice_yields_two_independent_receivers() {
        let bus = EventBus::new();
        let _rx1 = bus.subscribe();
        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }
}
