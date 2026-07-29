//! 注入容器：D-09 的 trait 反转在运行期的落点。
//!
//! ## 为什么是 `Arc<dyn …>` 而不是泛型参数
//!
//! 泛型 `S: FeedbackSource` 会把 `prism-engine` 的具体类型经 `S` 泄漏进
//! `prism-mcp` 的公开签名（乃至 axum service 的类型），并让
//! `StreamableHttpService::new` 的 `'static` factory 闭包难写；`Arc<dyn …>`
//! 之下 factory 只是 clone 一个 Arc，本 crate 的公开类型保持单一具体形态。
//!
//! ## bearer token 的归属
//!
//! **prism-mcp 既不生成也不存储 token**——它只持有调用方注入的那一份。
//! Phase 1 由测试注入；Phase 6 由 `prism-engine` 从钥匙串
//! （`docs/keychain-naming.md` 的 `mcp_bearer_token`）读出后注入。
//! 密钥的唯一入口仍是 `prism-llm`，本 crate 不引入任何 keyring 依赖。

use std::sync::Arc;

use prism_types::{CommentSink, FeedbackSource};

use crate::McpError;

/// MCP handler 可用的全部能力面 + 本次安装的 bearer token。
#[derive(Clone)]
pub struct McpDeps {
    pub feedback: Arc<dyn FeedbackSource>,
    pub comments: Arc<dyn CommentSink>,
    bearer: Arc<str>,
}

impl McpDeps {
    /// **可失败构造**：空 / 纯空白的 bearer 在这里就被拒（CR-03 纵深第一层）。
    ///
    /// 本 crate 既不生成也不存储 token（见文件头 10-15 行），它唯一能做的就是拒绝一个
    /// 装不出门禁的输入。这个检查刻意放在构造期而不是比较层：放在比较层意味着
    /// 「每次比较都要记得」，放在这里它是「造不出来」——`McpDeps` 一旦存在，
    /// 其 bearer 就保证非空。
    ///
    /// Phase 6 由 `prism-engine` 从钥匙串读出 token 后注入，届时
    /// 「读到空串」「首启时 token 尚未写入」「调用点一个 `unwrap_or_default()`」
    /// 三条现实路径都会精确命中这里。按 D-06（无 key 时应用照常启动），
    /// 那一侧应把 `Err` 降级为「MCP 服务不启动 + 一条 warn」，**不要** `unwrap()`。
    ///
    /// 纵深的第二层在 `middleware::constant_time_eq`：即便有人绕过本构造器造出空配置，
    /// 比较层也拒。两层各有自己的测试，互不代替。
    pub fn new(
        feedback: Arc<dyn FeedbackSource>,
        comments: Arc<dyn CommentSink>,
        bearer: impl Into<Arc<str>>,
    ) -> Result<Self, McpError> {
        // 归一化与判空必须落在**同一份字节**上：只用 `trim()` 判空、却存原值，
        // 造出的是一个「构造成功但永远比不中」的门禁。呈递侧的值来自 HTTP 头，
        // 尾随 OWS 在解析层就被剥掉了，于是配置里那个换行永远没有客户端能呈上——
        // **每一个**请求都被拒，且拒绝形态与伪造 token 完全相同（一条
        // `warn!("bearer token mismatch")`，与攻击者不可区分）。失败是静默且
        // fail-closed 的，诊断成本极高（01-REVIEW.md WR-06）。
        // 这不是理论路径：Phase 6 由 `prism-engine` 从钥匙串读出 token 注入，
        // 钥匙串往返、文件回退、`Command` 输出三条都常带尾随换行。
        let bearer: Arc<str> = Arc::from(bearer.into().trim());
        if bearer.is_empty() {
            return Err(McpError::EmptyBearer);
        }
        Ok(Self {
            feedback,
            comments,
            bearer,
        })
    }

    /// token 原文的唯一取用口。
    ///
    /// 刻意不是 `pub` 字段也刻意不叫 `bearer()`：调用点在代码搜索中可见，
    /// 且中间件之外没有第二个消费者。返回 `&str` 而非 `String` 是为了不制造副本。
    pub(crate) fn expose_bearer(&self) -> &str {
        &self.bearer
    }
}

/// 手写 `Debug`：token 绝不能经 `?deps` / `{:?}` 进日志或错误文本（T-01-29 同源要求）。
/// 与 `prism_llm::secrets::ApiKey` 同一口径——**刻意不实现 `Display`**，
/// 让 `format!("{deps}")` 成为编译错误而不是运行期泄漏。
impl std::fmt::Debug for McpDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpDeps")
            .field("feedback", &"<dyn FeedbackSource>")
            .field("comments", &"<dyn CommentSink>")
            .field("bearer", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_types::{FeedbackItem, Receipt, ServiceError};

    struct Empty;

    impl FeedbackSource for Empty {
        fn list_feedback(&self, _project_id: &str) -> Result<Vec<FeedbackItem>, ServiceError> {
            Ok(Vec::new())
        }
    }

    impl CommentSink for Empty {
        fn record_receipt(&self, _receipt: Receipt) -> Result<(), ServiceError> {
            Ok(())
        }
    }

    fn deps(bearer: &str) -> McpDeps {
        McpDeps::new(Arc::new(Empty), Arc::new(Empty), bearer)
            .expect("this helper is only called with a non-empty test bearer")
    }

    #[test]
    fn debug_does_not_reveal_the_bearer_token() {
        // 绑定名刻意不叫 `secret`：`scripts/check-secrets.sh` 抓的正是「名字像密钥的
        // 标识符后面跟一个引号串」这个形状，它没抓错。与 `prism-llm` 的 `FIXTURE_SECRET`
        // 和前端的 `FAKE_KEY` 同源——扫描器是防线，不该为了迁就测试局部变量而放宽。
        let fixture_bearer = "s3cr3t-token-value";
        let rendered = format!("{:?}", deps(fixture_bearer));
        assert!(
            !rendered.contains(fixture_bearer),
            "McpDeps Debug leaked the bearer token: {rendered}"
        );
        assert!(rendered.contains("<redacted>"), "{rendered}");
    }

    #[test]
    fn deps_are_cheap_to_clone_and_keep_the_same_token() {
        let d = deps("abc");
        let cloned = d.clone();
        assert_eq!(cloned.expose_bearer(), "abc");
    }

    /// CR-03 的纵深第一层：一个装不出门禁的输入在构造期就被拒。
    ///
    /// 「配置了但配了个空白」与「根本没配」是同一件事——Phase 6 从钥匙串读 token 时，
    /// 「读到空串」「首启时尚未写入」「调用点 `unwrap_or_default()`」三条路径都精确
    /// 命中这个形态。纵深的第二层在 `middleware::constant_time_eq`。
    #[test]
    fn an_empty_bearer_is_refused_at_construction() {
        // ① 空串：造不出来。
        let err = McpDeps::new(Arc::new(Empty), Arc::new(Empty), "")
            .expect_err("an empty bearer must not construct a gate");
        assert!(
            matches!(err, McpError::EmptyBearer),
            "wrong error variant for an empty bearer: {err:?}"
        );

        // ② 纯空白同样是「没配」。
        let blank = "    ";
        let err = McpDeps::new(Arc::new(Empty), Arc::new(Empty), blank)
            .expect_err("a whitespace-only bearer must not construct a gate");
        assert!(
            matches!(err, McpError::EmptyBearer),
            "wrong error variant for a blank bearer: {err:?}"
        );

        // ③ 形态纪律：错误文本不回显被拒的值，与手写 `Debug` 的 `<redacted>` 同源
        //    （T-01-29）。这里被拒的值恰好是空白串，所以断言的落点是「消息里没有它」。
        let rendered = err.to_string();
        assert!(
            !rendered.contains(blank),
            "the rejection echoed the value it refused: {rendered}"
        );
        assert!(
            !rendered.contains('"'),
            "the rejection quotes a value instead of stating a rule: {rendered}"
        );
        assert!(!rendered.is_empty(), "the rejection says nothing at all");

        // ④ 阴性对照：守卫不是「一律拒绝构造」。若写成一律 Err，①②③ 也都会绿。
        let fixture_bearer = "s3cr3t-bearer-value";
        let ok = McpDeps::new(Arc::new(Empty), Arc::new(Empty), fixture_bearer)
            .expect("a non-empty bearer must still construct");
        assert_eq!(
            ok.expose_bearer(),
            fixture_bearer,
            "the constructor altered a perfectly good token"
        );

        // ⑤ 归一化：判空与存值必须对**同一份字节**达成一致。只用 trim 判空却存原值，
        //    造出的是一个「构造成功但永远比不中」的门禁——呈递侧的尾随 OWS 已由 HTTP
        //    头解析层剥掉，配置侧那个换行则永远没人能呈递（01-REVIEW.md WR-06）。
        //    Phase 6 从钥匙串读 token，钥匙串往返 / 文件回退 / `Command` 输出都常带 `\n`。
        let padded = McpDeps::new(Arc::new(Empty), Arc::new(Empty), " tok ")
            .expect("a bearer with surrounding whitespace still carries a token");
        assert_eq!(
            padded.expose_bearer(),
            "tok",
            "the constructor stored the untrimmed value —— 门禁永远比不中"
        );

        // 归一化不得改动一个形态正常的 token：64 位十六进制、无外围空白，逐字返回。
        // （绑定名沿用 middleware.rs 的 `configured`：`scripts/check-secrets.sh` 抓的是
        //   「名字像密钥的标识符后跟引号串」这个形状，不该为迁就测试局部变量而放宽。）
        let configured = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let untouched = McpDeps::new(Arc::new(Empty), Arc::new(Empty), configured)
            .expect("a well-formed hex bearer must construct");
        assert_eq!(
            untouched.expose_bearer(),
            configured,
            "归一化改动了一个本来就干净的值"
        );

        let from_keychain = format!("{fixture_bearer}\n");
        let trailing_newline =
            McpDeps::new(Arc::new(Empty), Arc::new(Empty), from_keychain.as_str())
                .expect("a trailing newline must not build an unopenable gate");
        assert_eq!(
            trailing_newline.expose_bearer(),
            fixture_bearer,
            "尾随换行被原样存下 —— 呈递侧永远带不上它"
        );
    }
}
