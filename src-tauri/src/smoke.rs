//! Channel 有序流的样例数据源（Pattern 6）。
//!
//! Tauri 官方对事件系统的建议是「有序、高吞吐的数据投递请改用 Channel」，
//! 本模块就是那条通路的数据侧。它刻意**不依赖 tauri**：把「产生哪些事件」
//! 与「怎么把它们送出去」分开之后，有序性与空输入两条断言不需要真实 Channel、
//! 不需要 mock runtime 就能验证，而 [`crate::commands::dev_smoke_stream`]
//! 只剩下把它们逐条 `send` 这一件事。

use serde::Serialize;

/// 冒烟页默认的流长度。
///
/// 取 1000 而不是个位数：有序性只在「快速连发」下才可能被违反，`total = 3`
/// 的流即使实现是乱序的也大概率看起来是对的——那种绿证明不了任何事。
pub const SMOKE_DEFAULT_TOTAL: u32 = 1000;

/// 流事件。序列化形态是跨 IPC 边界的契约，与前端的判别联合一一对应。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SmokeEvent {
    Started { total: u32 },
    Tick { seq: u32 },
    Finished { total: u32 },
}

/// 按 `total` 生成一条流：`Started` 首、`total` 条 `Tick`（seq 由 0 递增）、`Finished` 末。
///
/// `sink` 的错误立即中止并原样返回——Channel 已关闭时继续 send 只是白白刷日志。
/// `total = 0` 是合法输入，产出 `[Started, Finished]` 两条，返回 `Ok`：
/// 空输入的正确语义是成功，不是错误、更不是挂起。
pub fn generate<E>(
    total: u32,
    mut sink: impl FnMut(SmokeEvent) -> Result<(), E>,
) -> Result<(), E> {
    sink(SmokeEvent::Started { total })?;
    for seq in 0..total {
        sink(SmokeEvent::Tick { seq })?;
    }
    sink(SmokeEvent::Finished { total })
}

/// [`generate`] 的收集形态，供不需要真实 Channel 的断言使用。
pub fn collect(total: u32) -> Vec<SmokeEvent> {
    let mut out = Vec::with_capacity(total as usize + 2);
    let Ok(()) = generate::<std::convert::Infallible>(total, |ev| {
        out.push(ev);
        Ok(())
    });
    out
}

#[cfg(test)]
mod tests {
    use super::{collect, generate, SmokeEvent, SMOKE_DEFAULT_TOTAL};

    /// 有序性断言的形态是**序列比较**，不是集合比较。
    ///
    /// `assert_eq!(seqs, 0..total)` 在乱序时会红；把两边各自 `sort()` 或塞进
    /// `HashSet` 再比就不会——后者只能证明「这些数都出现过」，而乱序正是
    /// 这条通路唯一要防的失败模式。
    #[test]
    fn smoke_stream_seq_is_strictly_monotonic() {
        let total = SMOKE_DEFAULT_TOTAL;
        let events = collect(total);

        assert_eq!(events.len() as u32, total + 2);
        assert_eq!(events.first(), Some(&SmokeEvent::Started { total }));
        assert_eq!(events.last(), Some(&SmokeEvent::Finished { total }));

        let seqs: Vec<u32> = events
            .iter()
            .filter_map(|ev| match ev {
                SmokeEvent::Tick { seq } => Some(*seq),
                _ => None,
            })
            .collect();

        assert_eq!(seqs, (0..total).collect::<Vec<u32>>());
        for pair in seqs.windows(2) {
            assert_eq!(pair[1], pair[0] + 1, "seq 出现缺口或重复: {pair:?}");
        }
    }

    #[test]
    fn smoke_stream_total_zero_emits_no_ticks() {
        assert_eq!(
            collect(0),
            vec![
                SmokeEvent::Started { total: 0 },
                SmokeEvent::Finished { total: 0 },
            ]
        );
        assert!(generate::<()>(0, |_| Ok(())).is_ok());
    }

    /// sink 的失败必须冒泡，不能被吞掉。
    ///
    /// 吞掉的表现是命令永远返回 `Ok`，而前端那边其实一条都没收到。
    #[test]
    fn smoke_stream_stops_at_the_first_sink_failure() {
        let mut seen = 0usize;
        let outcome = generate::<&str>(SMOKE_DEFAULT_TOTAL, |_| {
            seen += 1;
            if seen == 3 {
                Err("channel closed")
            } else {
                Ok(())
            }
        });

        assert_eq!(outcome, Err("channel closed"));
        assert_eq!(seen, 3);
    }
}
