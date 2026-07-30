//! `Engine` 门面：shell 与 MCP 之外的一切编排都发生在这里。
//!
//! **不依赖 tauri**（D-01）。shell 的每个 `#[tauri::command]` 应当是对本文件某个方法的
//! 单行委托——业务逻辑写进命令体是 ARCHITECTURE Anti-Pattern 1。
//!
//! # 三条纪律
//!
//! ## 1. 单写者句柄不外泄（T-01-31）
//!
//! `Engine` 持有 `Arc<Store>`，而**不提供任何返回 `Connection` / `&Connection` /
//! `PooledConnection` 的公开方法**。写路径统一是 `store.write(|tx| …)` 闭包，
//! 于是「拿一个连接自己写」在这套 API 下无从表达——Phase 2+ 的写路径不可能
//! 绕过单写者纪律，因为绕过的语法不存在。
//!
//! ## 2. 本文件的方法全部是**同步**的
//!
//! 底层 `prism-store`（rusqlite）与 `prism-llm`（keyring）本来就是阻塞的。
//! 从 async 上下文进入时，由**调用方**（shell 命令层 / MCP handler）用
//! `tokio::task::spawn_blocking` 包装。这不是把麻烦推给上层——它是让
//! `std::sync::MutexGuard` 的 `!Send` 继续在编译期挡住「跨 `.await` 持有写锁」：
//! 方法一旦变成 `async fn`，那个编译期保护就失效了。
//!
//! ## 3. 密钥只经 `prism_llm::secrets`（NFR-03）
//!
//! 本 crate 不声明也不 `use` `keyring_core` / `reqwest`——网络与密钥的实际出口
//! 唯一地在 `prism-llm` 内，facade 只做转交。`api_key_status` 返回**布尔**而不是
//! 密钥本身：门面不向上层返回密钥原文（T-01-04a）。

use std::sync::Arc;

use prism_store::settings::{self, SETTING_BASE_URL};
use prism_store::Store;
use prism_types::{EngineEvent, SearchHit};
use tokio::sync::broadcast::Receiver;

use crate::bus::EventBus;
use crate::error::EngineError;

/// 编排入口：单写者句柄的持有者 + 事件总线的唯一订阅点。
pub struct Engine {
    store: Arc<Store>,
    bus: EventBus,
}

impl Engine {
    pub fn new(store: Arc<Store>) -> Engine {
        Engine {
            store,
            bus: EventBus::new(),
        }
    }

    // ------------------------------------------------------------ 总线

    /// 新增一个事件订阅者。shell adapter（plan 01-08）在启动时调一次。
    pub fn subscribe(&self) -> Receiver<EngineEvent> {
        self.bus.subscribe()
    }

    /// 广播一条失效信号。无订阅者不是错误——见 [`crate::bus::EventBus::publish`]。
    pub fn publish(&self, ev: EngineEvent) {
        self.bus.publish(ev)
    }

    // ------------------------------------------------------------ 探针

    /// 端到端探针：把 store 报告的 SQLite 版本原样送回调用方。
    pub fn ping(&self) -> Result<String, EngineError> {
        Ok(self.store.sqlite_version()?)
    }

    pub fn types_crate_version(&self) -> &'static str {
        prism_types::CRATE_VERSION
    }

    // ------------------------------------------------------------ 搜索

    /// 在一个项目内全文搜索。
    ///
    /// 长度分流（≥3 字符 trigram MATCH / <3 字符 LIKE 回退）封装在 `prism-store`
    /// 内部，facade 不重复判断——两处各判一次必然漂移。
    pub fn search(&self, project_id: &str, q: &str) -> Result<Vec<SearchHit>, EngineError> {
        Ok(self.store.read(|c| prism_store::search(c, project_id, q))?)
    }

    /// 写入冒烟页的样例文档，返回它们所属的 project id。
    ///
    /// 返回 id 而不是让前端另存一份常量：两份常量必然漂移，而漂移的表现是
    /// 「播种成功但搜不到」——那正好长得像 FTS 坏了。
    ///
    /// 写入后广播 [`EngineEvent::Resync`]：一次播种改的是多份文档，
    /// 粗粒度失效语义下诚实的说法就是「你手上的东西全都作废，重取」。
    pub fn seed_sample_docs(&self) -> Result<String, EngineError> {
        self.store.write(prism_store::seed::insert_samples)?;
        self.publish(EngineEvent::Resync);
        Ok(prism_store::seed::SAMPLE_PROJECT_ID.to_string())
    }

    // ------------------------------------------------------------ 非密钥配置

    /// 读一条非密钥配置。查不到是 `Ok(None)`，不是错误。
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, EngineError> {
        Ok(self.store.read(|c| settings::get_setting(c, key))?)
    }

    /// 写 LLM 端点（D-05：非密钥配置进 `settings` 表，**不进钥匙串**）。
    ///
    /// 这里刻意**不**在写入前自己再调一次 `validate_base_url`：校验已经长在
    /// `set_setting` 内部（01-05 把它从调用方移进了写入路径，理由是"放调用方是约定，
    /// 放写入路径才是机制"）。facade 再抄一遍只会制造两份可能漂移的规则。
    ///
    /// 写入成功后广播一条 [`EngineEvent::Resync`]——notify-then-fetch 的粗粒度失效
    /// 语义下，配置变更的正确通知就是"你手上的东西全都作废，重取"。
    pub fn set_base_url(&self, raw: &str) -> Result<(), EngineError> {
        self.store
            .write(|tx| settings::set_setting(tx, SETTING_BASE_URL, raw))?;
        self.publish(EngineEvent::Resync);
        Ok(())
    }

    // ------------------------------------------------------------ 密钥（全部转交 prism-llm）

    /// 注册进程级钥匙串后端。**应用启动时调一次，且必须在任何密钥读写之前。**
    #[cfg(target_os = "macos")]
    pub fn init_secrets(&self) -> Result<(), EngineError> {
        Ok(prism_llm::secrets::init_default_store()?)
    }

    /// 写入 LLM API key。原文只在这一行上经过 facade，不落库、不进日志、不进错误文本。
    pub fn set_api_key(&self, secret: &str) -> Result<(), EngineError> {
        Ok(prism_llm::secrets::set_api_key(secret)?)
    }

    /// 是否已配置 API key。
    ///
    /// **返回布尔而不是密钥本身**（T-01-04a）：前端只需要知道"要不要显示未配置提示"，
    /// 把原文送上去就等于让它穿过 IPC 边界并可能进 WebView 的内存与日志。
    /// 无 key 是 `Ok(false)` 而不是 `Err`——D-06 要求无 key 时应用照常启动。
    pub fn api_key_status(&self) -> Result<bool, EngineError> {
        Ok(prism_llm::secrets::get_api_key()?.is_some())
    }

    /// 删除 API key。幂等：已不存在视为已删除。
    pub fn delete_api_key(&self) -> Result<(), EngineError> {
        Ok(prism_llm::secrets::delete_api_key()?)
    }
}

#[cfg(test)]
mod tests {
    /// 本文件的**生产代码**，去掉注释行与测试模块自身。
    ///
    /// 两处都必须去掉，否则下面的守卫会被自己的字面量和文档里的举例命中——
    /// 那样它就恒红（且看不出是被什么命中的），失去哨兵的意义。
    fn production_source() -> String {
        let source = include_str!("facade.rs");
        let cut = source.find("#[cfg(test)]").unwrap_or(source.len());
        source[..cut]
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 单写者句柄不外泄（T-01-31）的**源码级**哨兵。
    ///
    /// 为什么是源码断言而不是行为断言：一个"返回连接"的方法一旦存在，它本身就是
    /// 漏洞——没有哪次调用会失败，所以没有任何行为测试能红。能观测的只有 API 形状。
    #[test]
    fn no_public_method_hands_out_a_connection() {
        let source = production_source();
        for leak in [
            "-> Connection",
            "-> &Connection",
            "-> rusqlite::Connection",
            "PooledConnection",
        ] {
            assert!(
                !source.contains(leak),
                "facade 出现了外泄连接句柄的返回类型 {leak:?} —— 调用方可以绕过单写者纪律"
            );
        }
    }

    /// NFR-03 的源码级哨兵：密钥与网络只经 `prism-llm` 转交。
    ///
    /// `prism_llm::secrets` 必须出现（证明确实是转交而不是自己实现），
    /// `keyring_core` / `reqwest` / `apple_native_keyring_store` 必须不出现
    /// （证明 facade 没有开第二个出口）。依赖树层面的同一条性质由
    /// `scripts/check-deps.sh facade-egress` 守住，这里守的是源码层面。
    #[test]
    fn secrets_are_only_ever_delegated_to_prism_llm() {
        let source = production_source();
        assert!(
            source.contains("prism_llm::secrets"),
            "facade 没有把密钥读写转交给 prism-llm"
        );
        for direct in ["keyring_core", "apple_native_keyring_store", "reqwest::"] {
            assert!(
                !source.contains(direct),
                "facade 直接触碰了 {direct:?} —— 唯一密钥/网络出口不再唯一"
            );
        }
    }

    /// 门面方法必须是同步的（纪律 2）。
    ///
    /// 一旦有人写出 `pub async fn`，`std::sync::MutexGuard` 的 `!Send` 就不再能挡住
    /// 「跨 `.await` 持有写锁」——而那种 bug 表现为偶发 `SQLITE_BUSY`，最难查。
    #[test]
    fn facade_methods_are_synchronous() {
        let source = production_source();
        assert!(
            !source.contains("pub async fn"),
            "facade 出现了 async 方法：跨 await 持写锁的编译期保护随之失效"
        );
    }
}
