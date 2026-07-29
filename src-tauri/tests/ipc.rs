#![cfg(feature = "test")]
//! 进程内 IPC 测试：命令注册面与 Channel 通路。
//!
//! **首行那条 inner attribute 不是风格问题。** cargo 会编译 `tests/` 下的
//! 每一个文件，与命令行上的 `--test` / 过滤词无关；而 `tauri::test::mock_builder`
//! 只在非默认的 `test` feature 下存在。没有这行，本文件一落地就会让
//! `cargo test -p prismdocs-shell` 与 `cargo test --workspace` 在**编译期**失败。
//! 有了它，未开 feature 时整个文件编译为零个测试——代价是那两条命令
//! **不覆盖**本文件，唯一覆盖它的是 `cargo test -p prismdocs-shell --features test`。
//!
//! **能力边界：** mock runtime 没有真实 WebView，因此这里**证明不了「事件真的到达 JS」**。
//! 那半边由 plan 01-09 的冒烟页人工验证补齐。

use std::sync::Arc;

use prism_engine::Engine;
use prism_store::Store;
use prismdocs_shell::{commands, AppState};
use serde_json::json;
use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
use tauri::test::{mock_builder, MockRuntime, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{App, Manager, WebviewWindow, WebviewWindowBuilder};

/// 命令面的全集。`generate_handler!` 里注册了几个，这里就必须列几个。
const COMMANDS: [&str; 10] = [
    "dev_ping",
    "search_documents",
    "set_api_key",
    "api_key_status",
    "delete_api_key",
    "get_setting",
    "set_base_url",
    "dev_emit_bus_event",
    "dev_smoke_stream",
    "dev_seed_sample_docs",
];

/// 用临时目录的库装配一个 mock app。
///
/// **绝不碰真实 sidecar**（`~/Library/Application Support/PrismDocs/`）：
/// `AppState::bootstrap()` 走的是 `default_db_path()`，测试里直接构造 `AppState`。
fn mock_app() -> (App<MockRuntime>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(&dir.path().join("ipc.db")).expect("open store");

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
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
        ])
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("failed to build the mock app");

    app.manage(AppState {
        engine: Arc::new(Engine::new(Arc::new(store))),
    });

    (app, dir)
}

fn main_webview(app: &App<MockRuntime>) -> WebviewWindow<MockRuntime> {
    WebviewWindowBuilder::new(app, "main", Default::default())
        .build()
        .expect("failed to build the mock webview")
}

/// 请求来源。**必须与 `tauri.conf.json` 的 `devUrl` 一致。**
///
/// Tauri 用 `is_local_url` 判断来源是否本地：非本地来源会强制走 ACL，而本项目
/// 没有 capabilities 目录，于是每个命令都会被 `not allowed` 拒掉——那与「命令没注册」
/// 长得几乎一样。`http://tauri.localhost` 是 **Windows/Android** 上的自定义协议形态，
/// 在 macOS 上不是本地来源；真实 app 在 dev 下的来源就是这里的 devUrl。
const LOCAL_ORIGIN: &str = "http://localhost:1420";

fn invoke(
    webview: &WebviewWindow<MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<InvokeResponseBody, serde_json::Value> {
    tauri::test::get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: LOCAL_ORIGIN.parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
}

/// 每个命令的一份最小合法载荷。Channel 参数按 `__CHANNEL__:<id>` 的 IPC 形态传入。
fn payload(cmd: &str) -> serde_json::Value {
    match cmd {
        "search_documents" => json!({ "projectId": "p1", "q": "hello" }),
        "set_api_key" => json!({ "secret": "not-a-real-value" }),
        "get_setting" => json!({ "key": "llm.base_url" }),
        "set_base_url" => json!({ "url": "https://api.example.com" }),
        "dev_emit_bus_event" => json!({ "projectId": "p1", "docId": "d1" }),
        "dev_smoke_stream" => json!({ "onEvent": "__CHANNEL__:1", "total": 5 }),
        _ => json!({}),
    }
}

#[test]
fn smoke_stream_command_is_registered_and_returns_ok() {
    let (app, _dir) = mock_app();
    let webview = main_webview(&app);

    let res = invoke(&webview, "dev_smoke_stream", payload("dev_smoke_stream"));
    assert!(res.is_ok(), "dev_smoke_stream 未返回 Ok: {res:?}");
}

/// 不需要钥匙串的六个命令：本测试进程里它们必须**返回 Ok**。
///
/// 断言 Ok 而不是「错误不像未注册」，是因为后者对「命令注册了但委托写错了」不敏感。
const COMMANDS_EXPECTED_OK: [&str; 7] = [
    "dev_ping",
    "search_documents",
    "get_setting",
    "set_base_url",
    "dev_emit_bus_event",
    "dev_smoke_stream",
    "dev_seed_sample_docs",
];

/// 需要钥匙串的两个命令。
///
/// 本测试进程**从不调用** `init_secrets`，所以 `keyring_core` 没有默认后端，
/// 这两条会在触碰真实登录钥匙串之前就失败——测试因此不会弹授权框、CI 也不会挂。
const COMMANDS_NEEDING_KEYCHAIN: [&str; 3] =
    ["set_api_key", "api_key_status", "delete_api_key"];

/// 八个命令全部可经 IPC 到达，且错误串已被映射收敛。
///
/// **负对照是这个测试的判别性所在**：先用一个不存在的命令名确认「未注册」确实有
/// 可观测的错误形态（`Command X not found`），再断言八个真命令都不是那个形态。
/// 没有负对照的话，「错误串里没有 not found」在 marker 写错时也恒真——
/// 而这正是本测试第一版实际踩到的：来源 URL 写成 Windows 形态时每个命令都被 ACL
/// 拒成 `not allowed. Plugin not found`，与「未注册」肉眼难分。
#[test]
fn all_commands_are_registered() {
    let (app, _dir) = mock_app();
    let webview = main_webview(&app);

    let control = invoke(&webview, "definitely_not_a_command", json!({}))
        .expect_err("未注册的命令必须报错");
    let control = control.to_string();
    assert!(
        control.contains("not found"),
        "负对照失效：未注册命令的错误形态不是 `not found`，实得 {control}"
    );

    for cmd in COMMANDS {
        if let Err(err) = invoke(&webview, cmd, payload(cmd)) {
            let err = err.to_string();
            assert!(
                !err.contains("not found"),
                "命令 {cmd} 未注册（错误与负对照同形）: {err}"
            );
        }
    }

    for cmd in COMMANDS_EXPECTED_OK {
        let res = invoke(&webview, cmd, payload(cmd));
        assert!(res.is_ok(), "命令 {cmd} 的委托没走通: {res:?}");
    }

    // 错误路径同样受约束：钥匙串失败必须以**映射后的短码**出场，
    // 而不是 keyring 的平台错误原文（T-01-11）。
    for cmd in COMMANDS_NEEDING_KEYCHAIN {
        let err = invoke(&webview, cmd, payload(cmd))
            .expect_err("本进程没有钥匙串后端，这两条应当失败")
            .to_string();
        assert_eq!(
            err, "\"secret_error\"",
            "命令 {cmd} 的错误串没有经 map_err 收敛"
        );
    }
}
