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

/// 冒烟流的硬上界（T-01G-27）。
///
/// 每条 `Tick` 是一条 IPC 消息，而 `total` 来自 WebView 里的任意脚本——`u32::MAX`
/// 是 42.9 亿条。取值刻意留在 [`SMOKE_DEFAULT_TOTAL`] 之上一个数量级而不是贴着它：
/// 冒烟页固定用默认值，**正常路径必须不被夹紧**，否则「实收 1000 条」这条人工
/// 验证判据会失效。这个大小关系由下面那条编译期断言看住。
pub const SMOKE_MAX_TOTAL: u32 = 10_000;

/// 上界必须严格高于冒烟页固定使用的默认值。
///
/// 写成编译期断言而不是单测：把上界调到 1000 以下是一次编辑就能犯的错，而它的
/// 后果（人工验证判据静默失效）恰恰是「跑起来看着还挺正常」那一类。
const _: () = assert!(SMOKE_DEFAULT_TOTAL < SMOKE_MAX_TOTAL);

/// 流事件。序列化形态是跨 IPC 边界的契约，与前端的判别联合一一对应。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SmokeEvent {
    Started { total: u32 },
    Tick { seq: u32 },
    Finished { total: u32 },
}

/// `total` 的**唯一**夹紧点。
///
/// [`generate`] 与 [`collect`] 都经这里，上界的数字于是只有一处。夹紧放在本模块
/// 而不是调用方：本模块是这条流的真相源，且它不依赖 tauri——上界因此不需要真实
/// Channel 就能被断言。
fn clamp_total(total: u32) -> u32 {
    total.min(SMOKE_MAX_TOTAL)
}

/// 按 `total` 生成一条流：`Started` 首、`total` 条 `Tick`（seq 由 0 递增）、`Finished` 末。
///
/// `total` 先经 [`clamp_total`] 夹紧到 [`SMOKE_MAX_TOTAL`]，`Started` / `Finished`
/// 携带的也是夹紧后的值——流对自己保持自洽，否则前端「实收 N 条 == Started 报的
/// total」这条校验会在一条完好的流上报失败。
///
/// `sink` 的错误立即中止并原样返回——Channel 已关闭时继续 send 只是白白刷日志。
/// `total = 0` 是合法输入，产出 `[Started, Finished]` 两条，返回 `Ok`：
/// 空输入的正确语义是成功，不是错误、更不是挂起。
pub fn generate<E>(total: u32, mut sink: impl FnMut(SmokeEvent) -> Result<(), E>) -> Result<(), E> {
    let total = clamp_total(total);
    sink(SmokeEvent::Started { total })?;
    for seq in 0..total {
        sink(SmokeEvent::Tick { seq })?;
    }
    sink(SmokeEvent::Finished { total })
}

/// [`generate`] 的收集形态，供不需要真实 Channel 的断言使用。
pub fn collect(total: u32) -> Vec<SmokeEvent> {
    // 预分配按同一上界夹紧（T-01G-29）：未校验的 `total` 直接进 `with_capacity`
    // 意味着一次 `u32::MAX` 的调用先申请十几 GB，而循环连一条都还没产出。
    let mut out = Vec::with_capacity(clamp_total(total) as usize + 2);
    let Ok(()) = generate::<std::convert::Infallible>(total, |ev| {
        out.push(ev);
        Ok(())
    });
    out
}

#[cfg(test)]
mod tests {
    use super::{collect, generate, SmokeEvent, SMOKE_DEFAULT_TOTAL, SMOKE_MAX_TOTAL};

    fn tick_count(events: &[SmokeEvent]) -> u32 {
        events
            .iter()
            .filter(|ev| matches!(ev, SmokeEvent::Tick { .. }))
            .count() as u32
    }

    /// 上界必须真的把输入夹住（T-01G-27）。
    ///
    /// 用 `SMOKE_MAX_TOTAL + 1` 而不是 `u32::MAX`：前者足以证伪「没有夹紧」，
    /// 后者要跑 42.9 亿次循环才能给出同一个结论。
    #[test]
    fn smoke_stream_clamps_a_total_above_the_ceiling() {
        let events = collect(SMOKE_MAX_TOTAL + 1);

        assert_eq!(tick_count(&events), SMOKE_MAX_TOTAL);
        // 首末事件报出的也必须是夹紧后的值。否则流对自己不自洽：前端的
        // 「实收 N 条 == Started 报的 total」校验会在一条完好的流上报失败。
        assert_eq!(
            events.first(),
            Some(&SmokeEvent::Started {
                total: SMOKE_MAX_TOTAL
            })
        );
        assert_eq!(
            events.last(),
            Some(&SmokeEvent::Finished {
                total: SMOKE_MAX_TOTAL
            })
        );
    }

    /// 冒烟页固定使用的 1000 条路径**不**被夹紧。
    ///
    /// 与上一条成对：单看上一条，一个 `total.min(1)` 的实现也能绿——而那会让
    /// 冒烟页的「实收 1000 条」永远读不到 1000，人工验证判据就此失效。
    /// （大小关系本身由 `SMOKE_MAX_TOTAL` 旁边的编译期断言看住；这里看的是
    /// 夹紧**没有把默认路径的读数改掉**。）
    #[test]
    fn the_default_total_passes_through_unclamped() {
        assert_eq!(
            tick_count(&collect(SMOKE_DEFAULT_TOTAL)),
            SMOKE_DEFAULT_TOTAL
        );
    }

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
