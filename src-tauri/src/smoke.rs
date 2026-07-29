//! Channel 有序流的样例数据源（Pattern 6）。
//!
//! RED 阶段：本文件当前只有测试，`SmokeEvent` / `generate` / `collect` /
//! `SMOKE_DEFAULT_TOTAL` 尚不存在，编译即失败。

#[cfg(test)]
mod tests {
    use super::{collect, generate, SmokeEvent, SMOKE_DEFAULT_TOTAL};

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
            assert_eq!(pair[1], pair[0] + 1);
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
