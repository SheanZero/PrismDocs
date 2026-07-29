//! prism-engine 的跨 crate 集成层：总线语义、门面委托与 D-09 注入面。
//!
//! 五组测试对应 plan 01-07 的五条 `<behavior>` 加一条端到端注入：
//!
//! | 组 | 测试 | 守住的性质 |
//! |----|------|-----------|
//! | 总线交付 | `bus_delivers_to_subscriber` | publish 的事件真的到达订阅者 |
//! | 无订阅者 | `publish_with_no_subscriber_is_ok` | 「没人在听」不是调用方的错误 |
//! | 多订阅者 | `bus_broadcasts_to_multiple_subscribers` | 广播语义，不是竞争消费 |
//! | 门面委托 | `search_delegates_to_store` / `base_url_goes_to_settings_not_keychain` | 三条路径各自落到正确的下层 |
//! | trait 实现 | `engine_satisfies_service_traits`（Task 2） | `Arc<Engine>` 可注入真实 prism-mcp |
//!
//! ## 为什么每次 `recv` 都包 `timeout`
//!
//! 这是**反证落点**的要求，不是防御性编程。`broadcast::Receiver::recv()` 在 sender
//! 仍然存活时会一直等下去——若 `publish` 被改成空实现，不带 timeout 的测试会**挂住**
//! 而不是变红，反证就没有落点可看（STATE.md 记录的失效模式：反证要看红在哪一条断言）。
//! 包上 timeout 之后，「事件没送出去」会精确地落在下面那条具名的 `expect` 上。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use prism_engine::{Engine, EngineError, EventBus};
use prism_store::settings::SETTING_BASE_URL;
use prism_store::Store;
use prism_types::EngineEvent;
use serial_test::serial;
use tokio::sync::broadcast::Receiver;

/// 等一条事件的上限。本地进程内广播的实际延迟在微秒级；两秒纯粹是给
/// 「根本没送出去」一个确定的落点。
const RECV_TIMEOUT: Duration = Duration::from_secs(2);

const PROJECT: &str = "p1";
const DOC: &str = "d1";

/// 4 字中文词——长度 ≥ MIN_TRIGRAM_CHARS，走 trigram MATCH 分支。
const CJK_TERM: &str = "锚点迁移";
/// 库中不存在的词。阴性对照用：它必须返回 0 行。
const ABSENT_TERM: &str = "量子纠缠";

fn sample_event() -> EngineEvent {
    EngineEvent::DocChanged {
        project_id: PROJECT.to_string(),
        doc_id: DOC.to_string(),
    }
}

async fn recv_within(rx: &mut Receiver<EngineEvent>, who: &str) -> EngineEvent {
    tokio::time::timeout(RECV_TIMEOUT, rx.recv())
        .await
        .unwrap_or_else(|_| panic!("{who} 在 {RECV_TIMEOUT:?} 内没有收到事件——publish 没有把它送出去"))
        .unwrap_or_else(|e| panic!("{who} 的接收端异常关闭: {e}"))
}

// ---------------------------------------------------------------- 总线

#[tokio::test]
async fn bus_delivers_to_subscriber() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe();

    bus.publish(sample_event());

    let got = recv_within(&mut rx, "订阅者").await;
    assert_eq!(
        got,
        sample_event(),
        "订阅者收到的不是 publish 的那条事件"
    );
}

#[tokio::test]
async fn publish_with_no_subscriber_is_ok() {
    let bus = EventBus::new();
    assert_eq!(
        bus.subscriber_count(),
        0,
        "前置条件：这条测试要求总线上确实一个订阅者都没有"
    );

    // 返回类型必须是单元值：broadcast 在无接收端时 `send` 返回 Err，但对粗粒度失效
    // 信号而言「没人在听」不是错误——把它冒泡给调用方会让每条写路径都要处理一个
    // 无意义的错误分支。这一行同时是**编译期**断言：publish 若返回 Result 就编译不过。
    let outcome: () = bus.publish(sample_event());
    assert_eq!(outcome, (), "publish 不得向调用方返回错误");

    // 吞掉 SendError 不等于总线坏掉：之后来的订阅者照常收得到。
    let mut rx = bus.subscribe();
    bus.publish(sample_event());
    let got = recv_within(&mut rx, "无订阅者一轮之后才加入的订阅者").await;
    assert_eq!(got, sample_event());
}

#[tokio::test]
async fn bus_broadcasts_to_multiple_subscribers() {
    let bus = EventBus::new();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 2, "前置条件：两个独立订阅者");

    bus.publish(sample_event());

    let first = recv_within(&mut rx1, "第一个订阅者").await;
    // 竞争消费（mpsc 语义）下这一行会超时——广播语义要求同一条事件被送达每一个订阅者，
    // 而不是被第一个取走就没了。
    let second = recv_within(&mut rx2, "第二个订阅者").await;

    assert_eq!(first, second, "两个订阅者收到的应是同一条事件");
    assert_eq!(first, sample_event());
}

// ---------------------------------------------------------------- 门面委托

fn store_on_temp_db() -> (tempfile::TempDir, Arc<Store>) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let store = Store::open(&dir.path().join("prismdocs.db")).expect("open store");
    (dir, Arc::new(store))
}

/// 直接经 store 写一篇文档。
///
/// **测试自己持有 `Arc<Store>`**，而不是向 Engine 要一个连接句柄——facade 不暴露
/// 任何返回 `Connection` / `PooledConnection` 的方法（T-01-31），这里的写法正是
/// 那条纪律下调用方唯一能用的形态。
fn seed_document(store: &Store, title: &str, content: &str) {
    store
        .write(|tx| {
            tx.execute(
                "INSERT INTO projects(id, name, root_path, created_at) VALUES(?1, ?2, ?3, ?4)",
                (PROJECT, "fixture", "/tmp/fixture", 1_700_000_000_i64),
            )?;
            tx.execute(
                "INSERT INTO documents(id, project_id, rel_path, title, content, content_hash, updated_at) \
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                (
                    DOC,
                    PROJECT,
                    "docs/a.md",
                    title,
                    content,
                    "blake3-fixture",
                    1_700_000_000_i64,
                ),
            )?;
            Ok(())
        })
        .expect("seed the fixture document");
}

#[test]
fn search_delegates_to_store() {
    let (_dir, store) = store_on_temp_db();
    seed_document(&store, "锚点迁移设计说明", "trigram 索引下的中文命中");
    let engine = Engine::new(Arc::clone(&store));

    let hits = engine.search(PROJECT, CJK_TERM).expect("search");
    assert_eq!(
        hits.len(),
        1,
        "engine.search 应把查询转交给 store 的 FTS 索引，实得 {hits:?}"
    );
    assert_eq!(hits[0].doc_id, DOC);

    // 阴性对照：不做这一条，一个「返回硬编码结果」的 search 也能让上面全绿。
    let none = engine.search(PROJECT, ABSENT_TERM).expect("search");
    assert!(
        none.is_empty(),
        "库中不存在的词返回了 {none:?} —— search 不是真的在查 store"
    );
}

/// 装上进程级 mock 钥匙串。
///
/// 先 unset 再 set：前一个 `#[serial]` 测试若在收尾前 panic，默认 store 会残留下来
/// （沿用 01-04 在 prism-llm 建立的口径）。
fn install_mock_keychain() {
    let _ = keyring_core::unset_default_store();
    keyring_core::set_default_store(
        keyring_core::mock::Store::new().expect("mock store is constructible"),
    );
}

/// 当前 mock 钥匙串里属于 PrismDocs 的条目数。
fn keychain_entry_count() -> usize {
    let spec = HashMap::from([("service", prism_llm::secrets::SERVICE)]);
    keyring_core::Entry::search(&spec)
        .expect("search the default store")
        .len()
}

#[test]
#[serial]
fn base_url_goes_to_settings_not_keychain() {
    install_mock_keychain();
    let (_dir, store) = store_on_temp_db();
    let engine = Engine::new(store);
    let mut rx = engine.subscribe();

    assert_eq!(keychain_entry_count(), 0, "前置条件：钥匙串起点为空");

    engine
        .set_base_url("https://api.example.com/v1")
        .expect("a valid https endpoint should be accepted");

    // ① 值落在 settings 表里，读得回来。
    assert_eq!(
        engine
            .get_setting(SETTING_BASE_URL)
            .expect("get_setting")
            .as_deref(),
        Some("https://api.example.com/v1"),
        "base_url 没有落进 settings 表"
    );

    // ② 钥匙串里一条新增条目都没有——非密钥配置不得误走 keyring（D-05）。
    assert_eq!(
        keychain_entry_count(),
        0,
        "写 base_url 在钥匙串里留下了条目：非密钥配置走错了下层"
    );

    // ③ 写成功后广播一次失效信号（notify-then-fetch：只发信号，不发载荷）。
    let published = rx
        .try_recv()
        .expect("set_base_url 成功后应向总线发一条失效信号");
    assert!(
        matches!(published, EngineEvent::Resync),
        "settings 变更应发全量失效信号，实得 {published:?}"
    );

    // ④ 校验仍长在写入路径上：facade 不得成为绕过 store 守卫的旁路。
    let rejected = engine.set_base_url("file:///etc/passwd");
    assert!(
        matches!(rejected, Err(EngineError::Store(_))),
        "非法 scheme 应被拒绝，实得 {rejected:?}"
    );

    let _ = keyring_core::unset_default_store();
}
