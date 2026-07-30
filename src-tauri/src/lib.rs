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

/// 默认过滤档位。三条安全决策的日志（`middleware.rs` 的无差别 403 真实原因、
/// `settings.rs` 的明文 http 告警、下面那条钥匙串不可用降级提示）都是 `warn!`，
/// `info` 已经全部覆盖；刻意**不**给 prism_mcp 单开 `debug`——少开一个 target 的 debug，
/// 就少一份「日志里到底会出现什么」的不确定性（T-01-58：这个 sink 本身是新增的外泄面）。
///
/// 默认档位只管住「没人动 `RUST_LOG` 时」这一半。另一半是：这个 sink 还需要一个
/// **环境变量打不破的天花板**，见 [`LOG_CEILING_DIRECTIVE`]。
const DEFAULT_LOG_FILTER: &str = "info";

/// 项目日志上限：钉在环境提供的 filter 之上，`RUST_LOG` 抬不动它。
///
/// 为什么单点名 `rmcp`：`rmcp-2.2.0` 的 streamable http server
/// （`transport/streamable_http_server/tower.rs`）里有 `tracing::trace!(?message)`，
/// 即**完整 MCP 消息转储**。Phase 5 起那些消息载有评论正文与文档摘录——正是
/// `prism_engine::services::record_receipt` 刻意不记的东西（T-01-33）。
/// 于是 `DEFAULT_LOG_FILTER` 上方那段「少开一个 target 的 debug」的克制，
/// 会被一条 `export RUST_LOG=trace`、或某人 shell profile 里一条遗留的 `RUST_LOG`
/// 整个抵消掉（T-01G-22）。
///
/// 上限是**针对性**的而不是把所有 target 压回 `info`：开发者调 `prism_*` 的能力
/// 不该因为一条外泄面而被剥夺。
const LOG_CEILING_DIRECTIVE: &str = "rmcp=info";

/// 降档发生时那条 `warn!` 的正文。
///
/// 只陈述**规则**，不回显 `RUST_LOG` 的原值：环境变量里可能带着与本项目无关的
/// 别的东西，把它抄进日志等于用一条防外泄的告警制造一次外泄（T-01G-26）。
const LOG_CEILING_WARNING: &str =
    "the environment-supplied log filter exceeds the project ceiling; \
     the `rmcp` target was capped at INFO because raising it dumps whole MCP message bodies \
     into the local log sink";

/// 造一个只用于「问某个 filter 会不会放行」的 `Metadata` 常量。
///
/// 刻意**不**走 `tracing::event_enabled!`：那个宏问的是进程当前的全局 dispatcher，
/// 而且答案会经 callsite 兴趣缓存跨 subscriber 复用——判定结果于是取决于
/// 「这个测试二进制里此前装过什么」，正是本 plan 要拆掉的那类前置依赖。
/// 这里要问的是**手上这一个** filter，所以自造 callsite、自造 metadata，
/// 直接调 `Subscriber::enabled`，不碰任何全局状态。
macro_rules! log_filter_probe {
    ($name:ident, $target:expr, $level:expr) => {
        static $name: tracing::Metadata<'static> = {
            struct Probe;
            impl tracing::callsite::Callsite for Probe {
                fn set_interest(&self, _: tracing::subscriber::Interest) {}
                fn metadata(&self) -> &tracing::Metadata<'_> {
                    &$name
                }
            }
            static PROBE: Probe = Probe;
            tracing::Metadata::new(
                "log_filter_probe",
                $target,
                $level,
                None,
                None,
                None,
                // 等价于 `tracing::identify_callsite!(&PROBE)`——那个宏在 `tracing` 门面里
                // 是私有的（它只从 `tracing_core` 通配再导出），而本 crate 刻意不直接依赖
                // `tracing_core`，所以这里直接构造 `Identifier`。
                tracing::field::FieldSet::new(&[], tracing::callsite::Identifier(&PROBE)),
                tracing::metadata::Kind::EVENT,
            )
        };
    };
}

// `DEBUG` 是紧挨在上限 `INFO` 之上的那一档：filter 放行它，就等于上限没起作用。
log_filter_probe!(RMCP_DEBUG_PROBE, "rmcp", tracing::Level::DEBUG);

/// 手上这个 filter 是否放 `rmcp` 到上限之上。
fn allows_rmcp_above_ceiling(filter: tracing_subscriber::EnvFilter) -> bool {
    use tracing::Subscriber as _;
    use tracing_subscriber::layer::SubscriberExt as _;

    tracing_subscriber::registry()
        .with(filter)
        .enabled(&RMCP_DEBUG_PROBE)
}

/// 读环境提供的 filter；读不到或解析失败时回落到 [`DEFAULT_LOG_FILTER`]。
fn env_log_filter() -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(DEFAULT_LOG_FILTER))
}

/// 构建最终 filter，并报告是否发生了降档。
///
/// 「环境优先 + 追加封顶」而不是「超限就整体回落到默认档」：后者一刀切掉开发者
/// 调试别的 target 的能力。实测确认过追加的那一条压得住环境里的同名指令——`EnvFilter`
/// 按**特异性**（target 长度 / 字段数）而非书写顺序挑选生效指令，带 target 的
/// `rmcp=info` 比 `RUST_LOG=trace` 那条无 target 的全局指令更特异；而 `RUST_LOG=rmcp=trace`
/// 那种同 target 同字段的指令会被直接替换掉。两种形态都由下面的单测钉住。
///
/// `EnvFilter` 不实现 `Clone`，所以探针那一份从环境重新构建——`RUST_LOG` 在这两行之间
/// 不会变，两份等价。
fn build_log_filter() -> (tracing_subscriber::EnvFilter, bool) {
    let capped = allows_rmcp_above_ceiling(env_log_filter());

    let filter = env_log_filter().add_directive(
        LOG_CEILING_DIRECTIVE
            .parse()
            .expect("LOG_CEILING_DIRECTIVE is a compile-time constant"),
    );

    (filter, capped)
}

/// 安装全局 tracing subscriber。返回 `true` 表示**本次调用**完成了安装。
///
/// 收尾用 `try_init()` 而不是 `init()`：后者在全局 dispatcher 已就位时会 panic，
/// 而「装日志这件事本身把应用弄崩」是最不该发生的失败模式。返回 bool 让调用方与测试
/// 都能分辨「这次装上了」与「早就装好了」。
///
/// 过滤档位见 [`build_log_filter`]。降档告警刻意排在 `try_init()` **之后**——
/// subscriber 没装上之前发的 `warn!` 写向虚无，那条告警也就等于没发。
pub fn init_tracing() -> bool {
    let (filter, capped) = build_log_filter();

    let installed = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .is_ok();

    if capped {
        warn_log_ceiling_applied();
    }

    installed
}

fn warn_log_ceiling_applied() {
    tracing::warn!("{LOG_CEILING_WARNING}");
}

pub fn run() {
    // 必须是第一步：装在 `tauri::Builder` 之后就漏掉了 `AppState::bootstrap()` 的失败
    // 与下面「钥匙串不可用」那条降级提示。返回值显式丢弃——装不上不该阻断启动，
    // 与「钥匙串后端注册失败不阻断启动」同一口径。
    let _ = init_tracing();

    let builder = tauri::Builder::default().setup(|app| {
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
    });

    // 命令面按构建形态分叉，**编译期**而不是运行期。
    //
    // 前端门控管不了这件事：`App.tsx` 的 `import.meta.env.DEV` 摇掉的是 dev **路由按钮**，
    // 不是 dev **命令**。任何在 WebView 里执行的脚本——Phase 3+ 起要渲染外部 agent 写的
    // Markdown，CSP 是第一道，命令面最小化是第二道——都能直接
    // `invoke("dev_seed_sample_docs")`，往用户真实的
    // `~/Library/Application Support/PrismDocs/prismdocs.db` 里写三份 fixture 文档；
    // 那些行的 project id 是硬编码的 `smoke-project`（`prism_store::seed`），
    // 与真实导入在 schema 层不可区分，Phase 2 时还在那里（T-01G-23）。
    // 另一条 `dev_emit_bus_event` 可被刷成 UI 失效风暴（T-01G-24）。
    //
    // 不用运行期 `if`：那意味着命令**仍在二进制里**、仍在 ACL 之外的路径上可触及，
    // 而这里要的是「不存在」。
    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::dev_ping,
        commands::search_documents,
        commands::set_api_key,
        commands::api_key_status,
        commands::delete_api_key,
        commands::get_setting,
        commands::set_base_url,
        commands::dev_emit_bus_event,
        commands::dev_smoke_stream,
        commands::dev_seed_sample_docs,
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::search_documents,
        commands::set_api_key,
        commands::api_key_status,
        commands::delete_api_key,
        commands::get_setting,
        commands::set_base_url,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("failed to start the PrismDocs shell");
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serial_test::serial;

    use super::*;

    // 上限档位本身仍必须放行——「针对性封顶」不等于「把 rmcp 关掉」。
    log_filter_probe!(RMCP_INFO_PROBE, "rmcp", tracing::Level::INFO);
    // 项目自己的 target：上限**不**施加于它，否则「开发者要调试的能力不被剥夺」这句话失效。
    log_filter_probe!(ENGINE_TRACE_PROBE, "prism_mcp", tracing::Level::TRACE);
    log_filter_probe!(ENGINE_DEBUG_PROBE, "prism_mcp", tracing::Level::DEBUG);
    log_filter_probe!(ENGINE_INFO_PROBE, "prism_mcp", tracing::Level::INFO);

    /// `RUST_LOG` 是进程级状态。设了就得还，还得在 panic 路径上也还——所以是 `Drop`。
    struct RustLogGuard(Option<String>);

    impl RustLogGuard {
        fn set(value: &str) -> Self {
            let guard = Self(std::env::var("RUST_LOG").ok());
            std::env::set_var("RUST_LOG", value);
            guard
        }

        fn unset() -> Self {
            let guard = Self(std::env::var("RUST_LOG").ok());
            std::env::remove_var("RUST_LOG");
            guard
        }
    }

    impl Drop for RustLogGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(previous) => std::env::set_var("RUST_LOG", previous),
                None => std::env::remove_var("RUST_LOG"),
            }
        }
    }

    /// 此刻眼前这个 dispatcher 是不是一个**真的** subscriber。
    ///
    /// 不能用 `has_been_set()` 代替：那个标志由 `set_global_default` 与
    /// `with_default` 共同置真，且**永不复位**——本文件那条捕获日志的测试用
    /// `with_default` 装过一个线程局部的 subscriber，此后 `has_been_set()` 恒为真，
    /// 哪怕全局槽位仍然空着。
    fn a_real_subscriber_is_in_place() -> bool {
        tracing::dispatcher::get_default(|dispatch| {
            !dispatch.is::<tracing::subscriber::NoSubscriber>()
        })
    }

    /// 直接问 filter：这条 metadata 放不放行。不经全局 dispatcher，见 `log_filter_probe!`。
    fn probe(filter: tracing_subscriber::EnvFilter, meta: &tracing::Metadata<'static>) -> bool {
        use tracing::Subscriber as _;
        use tracing_subscriber::layer::SubscriberExt as _;

        tracing_subscriber::registry().with(filter).enabled(meta)
    }

    /// `RUST_LOG=trace` 抬不动 `rmcp`，但抬得动别的 target。
    ///
    /// 两条断言缺一不可：只留第一条的话，「把所有 target 都压回 `info`」也能让它绿，
    /// 而那种实现会让 [`LOG_CEILING_DIRECTIVE`] 文档里「上限是针对性的」那句话失效。
    #[test]
    #[serial]
    fn the_project_ceiling_holds_against_a_global_rust_log_trace() {
        let _guard = RustLogGuard::set("trace");

        let (filter, capped) = build_log_filter();
        assert!(
            capped,
            "RUST_LOG=trace 超出项目上限，降档这件事必须被报告出来"
        );

        assert!(
            !probe(filter, &RMCP_DEBUG_PROBE),
            "RUST_LOG=trace 把 rmcp 放行到了 DEBUG —— 整条 MCP 消息会被倒进本地日志"
        );

        assert!(
            probe(build_log_filter().0, &RMCP_INFO_PROBE),
            "上限档位本身被误伤：rmcp 的 INFO 事件也不再放行"
        );
        assert!(
            probe(build_log_filter().0, &ENGINE_TRACE_PROBE),
            "上限不该是全局的：prism_mcp 在 RUST_LOG=trace 下仍应可达 TRACE"
        );
    }

    /// 同 target 的显式指令同样压得住——`RUST_LOG=rmcp=trace` 是绕过全局档位的最短路径。
    #[test]
    #[serial]
    fn the_project_ceiling_replaces_a_target_specific_rust_log_directive() {
        let _guard = RustLogGuard::set("rmcp=trace");

        let (filter, capped) = build_log_filter();
        assert!(capped, "RUST_LOG=rmcp=trace 超出项目上限，降档必须被报告");
        assert!(
            !probe(filter, &RMCP_DEBUG_PROBE),
            "环境里的 rmcp=trace 覆盖掉了项目上限指令"
        );
    }

    /// 无 `RUST_LOG` 时，最终 filter 与 [`DEFAULT_LOG_FILTER`] 行为等价，且不报降档。
    #[test]
    #[serial]
    fn without_rust_log_the_filter_is_equivalent_to_the_default_filter() {
        let _guard = RustLogGuard::unset();

        let (filter, capped) = build_log_filter();
        assert!(!capped, "没有 RUST_LOG 时不存在降档，不该发那条 warn");

        assert!(probe(filter, &ENGINE_INFO_PROBE), "默认档 info 应放行 INFO");
        assert!(
            !probe(build_log_filter().0, &ENGINE_DEBUG_PROBE),
            "默认档 info 不该放行 DEBUG"
        );
        assert!(
            probe(build_log_filter().0, &RMCP_INFO_PROBE),
            "默认档下 rmcp 的 INFO 仍应放行"
        );
        assert!(
            !probe(build_log_filter().0, &RMCP_DEBUG_PROBE),
            "默认档下 rmcp 不该放行 DEBUG"
        );
    }

    #[derive(Clone, Default)]
    struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for CaptureWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("capture buffer")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl tracing_subscriber::fmt::MakeWriter<'_> for CaptureWriter {
        type Writer = CaptureWriter;

        fn make_writer(&self) -> Self::Writer {
            self.clone()
        }
    }

    /// 降档告警只陈述规则，不回显 `RUST_LOG` 原值（T-01G-26）。
    ///
    /// 第一条断言是负对照：没有它，「输出里不含 marker」在这条 warn 压根没被捕获到时
    /// 也恒真——而「没捕获到」正是 subscriber 装配写错时最可能的形态。
    #[test]
    #[serial]
    fn the_downgrade_warning_does_not_echo_the_rust_log_value() {
        const MARKER: &str = "leak_marker_must_not_reach_the_log";

        let _guard = RustLogGuard::set(&format!("trace,{MARKER}=trace"));

        let sink = CaptureWriter::default();
        let captured = Arc::clone(&sink.0);

        let subscriber = tracing_subscriber::fmt()
            .with_writer(sink)
            .with_ansi(false)
            .without_time()
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let (_filter, capped) = build_log_filter();
            assert!(capped, "RUST_LOG=trace 必须触发降档");
            warn_log_ceiling_applied();
        });

        let output = String::from_utf8(captured.lock().expect("capture buffer").clone())
            .expect("log output should be utf-8");

        assert!(
            output.contains("capped at INFO"),
            "负对照失效：那条降档 warn 根本没被捕获到，实得 {output:?}"
        );
        assert!(
            !output.contains(MARKER),
            "降档 warn 回显了 RUST_LOG 的原值，实得 {output:?}"
        );
    }

    /// 装上 subscriber 这件事本身没有任何行为面的观察点——workspace 里 7 处 `tracing`
    /// 发射点在没有 subscriber 时**照常编译、照常执行、照常返回**，只是写向虚无。
    /// 于是判别性只能落在 ②：全局 dispatcher 是否真的就位。
    ///
    /// 刻意**不写**「调用前 `has_been_set()` 为 false」这条前置断言——全局 dispatcher 是
    /// 进程级的，把前置条件写进判别性测试会让反证落在前置条件上而不是被守的那条断言上
    /// （本 phase 已记录的教训）。
    ///
    /// 同一条推理的另一半（IN-02）：断言 ① 原先写作「首次调用返回 true」，那本身就是
    /// 一条**隐式**前置条件——「本二进制里我是第一个装 subscriber 的」。它今天成立只因为
    /// 恰好没有第二处装；有人为了断言日志输出加一处，① 就开始指着错误的代码变红。
    ///
    /// 收口分两步，缺一不可（执行本 plan 时实测过）：
    ///
    /// 1. `#[serial]` 只挡**并发**，挡不住**顺序**。实测：在本文件加一条名字排在前面、
    ///    第一行调 `init_tracing()` 的测试，无论它标不标 `#[serial]`，原来的 ① 都是
    ///    `--test-threads=4` 下连跑五次五次全红——因为那条测试先跑完就把 dispatcher 装上了。
    /// 2. 所以 ① 改成按「调用前是否已就位」参数化：`init_tracing()` 的返回值必须报告
    ///    **这次**装没装上，而不是一个常量。这条不依赖调用顺序，判别力也没丢——桩实现
    ///    （恒返回 true 或恒返回 false）在两种世界里各有一种会被它逮住，② ③ 兜住其余。
    ///
    /// `#[serial]` 仍然要留：`has_been_set()` 与紧随其后的 `init_tracing()` 之间有一个
    /// 读-改窗口，而 Task 1 那四条 filter 测试还会改 `RUST_LOG`——`init_tracing()` 读它。
    /// **约定**：本 crate 里任何会装 subscriber 或改 `RUST_LOG` 的测试都必须进这个串行组。
    #[test]
    #[serial]
    fn tracing_init_installs_a_global_subscriber_and_is_idempotent() {
        // ① 返回值报告的是「本次调用装没装上」，不是一个常量
        let was_installed_before = a_real_subscriber_is_in_place();
        assert_eq!(
            init_tracing(),
            !was_installed_before,
            "init_tracing() 的返回值与「调用前全局 dispatcher 是否已就位」不一致"
        );

        // ② 判别性所在：若 init_tracing 被写成什么都不做的桩，① 仍可能绿而这条一定红
        assert!(
            tracing::dispatcher::has_been_set(),
            "no global dispatcher after init_tracing() — the 7 tracing call sites \
             across the workspace are still writing into a null sink"
        );

        // ②′ `has_been_set()` 单独已不足以支撑 ② 想说的那件事——见
        // `a_real_subscriber_is_in_place` 的注释。补这一句把 ② 拉回原本的强度。
        assert!(
            a_real_subscriber_is_in_place(),
            "the global dispatcher is still NoSubscriber after init_tracing()"
        );

        // ③ 重复调用安全：用 try_init 而非会 panic 的 init
        assert!(
            !init_tracing(),
            "the second init_tracing() should report that it did not install"
        );
    }

    /// `init_tracing()` 必须排在 `tauri::Builder` 之前。
    ///
    /// 这条断言看的是源码顺序，不是行为——`run()` 会阻塞直到窗口关闭，测不了。
    /// 顺序本身重要：装在 `Builder` 之后就漏掉了 `AppState::bootstrap()` 的失败与
    /// 「钥匙串不可用」那条降级提示，而那两条正是 WR-04 点名「无处可去」的日志。
    ///
    /// 定位刻意从 `pub fn run()` 起切片而不是全文 `find`：函数签名
    /// `pub fn init_tracing() -> bool` 本身就含 `init_tracing()` 这个子串，
    /// 全文 find 会永远命中定义处（它在 `run()` 之前），使断言恒真。
    ///
    /// 两个锚点都取**完整语句**而不是裸名字。执行本 plan 时实测过：锚点写成
    /// `"tauri::Builder"` 会命中 `run()` 里那条**解释性注释**中的 `tauri::Builder`
    /// 字样——它排在真正的调用之前，于是断言在实现完全正确时变红。源码序断言的
    /// 匹配面里同时含代码与注释，锚点必须窄到只有语句本身能命中。
    #[test]
    fn run_installs_tracing_before_it_builds_the_app() {
        let source = include_str!("lib.rs");
        let run_at = source
            .find("pub fn run()")
            .expect("lib.rs should declare pub fn run()");
        let body = &source[run_at..];

        let init_at = body
            .find("let _ = init_tracing();")
            .expect("run() should call init_tracing()");
        let builder_at = body
            .find("tauri::Builder::default()")
            .expect("run() should build the tauri app");

        assert!(
            init_at < builder_at,
            "init_tracing() runs after tauri::Builder — AppState::bootstrap() failures and \
             the keychain-unavailable degradation notice would both be lost"
        );
    }

    /// 发布形态的 IPC 面里不含四条 `dev_*` 命令。
    ///
    /// 为什么是源码断言：`cargo test` 跑的永远是 debug，两份命令列表里被编译进
    /// 测试二进制的只有 debug 那一份——release 那一份的内容在行为面上**测不出来**。
    /// 与 `run_installs_tracing_before_it_builds_the_app` 同一形态、同一理由。
    ///
    /// 锚点取**完整语句片段**而不是裸名字：`dev_seed_sample_docs` 这个裸名字在
    /// 分叉上方那段解释性注释里也出现，而源码序断言的匹配面里同时含代码与注释
    /// （本 phase 已实测过这会让断言假红）。切片同样必须收在该语句的 `]);` 上，
    /// 否则会一路吃到文件尾，把 debug 那一支和本测试自己的字符串字面量都算进来。
    ///
    /// 起点锚点跨行，于是它在本文件源码里只有一处能命中——本测试自己那份是**转义后的**
    /// 字面量（`\n` 是两个字符），与 `run()` 里真正的换行不同形。写本测试时实测过：
    /// 锚点只写单行的 `#[cfg(not(...))]` 时，`find` 会先命中本测试自己的字面量，
    /// 于是在实现根本没写分叉时也能走过 `expect`——判别力全靠下面那条负对照捞回来。
    #[test]
    fn the_release_ipc_surface_excludes_the_dev_commands() {
        const RELEASE_ARM_ANCHOR: &str =
            "#[cfg(not(debug_assertions))]\n    let builder = builder.invoke_handler(";

        let source = include_str!("lib.rs");
        let run_at = source
            .find("pub fn run()")
            .expect("lib.rs should declare pub fn run()");
        let body = &source[run_at..];

        let arm_at = body
            .find(RELEASE_ARM_ANCHOR)
            .expect("run() should register a release-only handler list");
        let arm = &body[arm_at..];
        let arm_end = arm
            .find("]);")
            .expect("the release arm should close a generate_handler! list");
        let arm = &arm[..arm_end];

        for dev in [
            "commands::dev_ping,",
            "commands::dev_emit_bus_event,",
            "commands::dev_smoke_stream,",
            "commands::dev_seed_sample_docs,",
        ] {
            assert!(
                !arm.contains(dev),
                "release 的 IPC 面上仍注册着 {dev} —— WebView 里任何脚本都能 invoke 它"
            );
        }

        // 非恒真的另一半：切出来的这一段确实是一份命令列表，而不是被切空的片段。
        // 少了这两条，把整个分叉删掉也能让上面的循环全绿。
        for production in ["commands::search_documents,", "commands::set_api_key,"] {
            assert!(
                !arm.is_empty() && arm.contains(production),
                "release 的命令列表里没有生产命令 {production} —— 切片锚点已失效"
            );
        }
    }
}
