//! 非密钥配置的 k/v 读写（D-05）。
//!
//! # 什么进得来，什么进不来
//!
//! 进得来：`base_url`、模型标识、项目阈值这类**非密钥**配置。存 SQLite 的三条理由是
//! 随库整体备份、单一真相源、与其余写入事务一致。
//!
//! 进不来：任何密钥。API key 与 MCP bearer token 走系统钥匙串（`prism-llm::secrets`），
//! **绝不入库**——这张表会随 sidecar 目录被整体备份带走，一次备份就把密钥送出了本机。
//! 这条边界由 [`is_secret_like_key`] 在写入路径上强制，不是靠注释约定：注释拦不住
//! 三个月后那个「先塞进去回头再改」的调用点。

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, Transaction};
use url::Url;

use crate::error::StoreError;

/// LLM 端点。写入前必过 [`validate_base_url`]。
pub const SETTING_BASE_URL: &str = "llm.base_url";
/// 模型标识。
pub const SETTING_MODEL: &str = "llm.model";

/// `base_url` 允许的 scheme。`file:` / `javascript:` 之类既是 SSRF 雏形也是本地文件读取面。
pub const ALLOWED_URL_SCHEMES: [&str; 2] = ["http", "https"];

/// 键名里出现即判定为疑似密钥的子串（大小写不敏感）。
const SECRET_KEY_MARKERS: [&str; 3] = ["key", "token", "secret"];

const SQL_GET: &str = "SELECT value FROM settings WHERE key = ?1";
const SQL_UPSERT: &str = "INSERT INTO settings(key, value, updated_at) VALUES(?1, ?2, ?3) \
     ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at";

/// 键名是否疑似密钥。
///
/// 宽进严出：宁可误拒一个叫 `ui.hotkey` 的正常配置（改名即可），
/// 也不放过一个叫 `llm.api_key` 的真密钥（一旦进库，备份就带得走）。
pub fn is_secret_like_key(key: &str) -> bool {
    let lowered = key.to_lowercase();
    SECRET_KEY_MARKERS.iter().any(|m| lowered.contains(m))
}

/// 解析并校验 LLM 端点。
///
/// 拒绝四类：不被允许的 scheme、空 host、userinfo 里的凭据、query 或 fragment。
/// 后两类是**值侧**守卫：[`is_secret_like_key`] 防的是键名，而 `llm.base_url`
/// 这个键名完全正常，凭据藏在值里（01-VERIFICATION.md gap 1）。
///
/// 错误里只有键名与规则，**没有传入的 value**——被误填进这里的很可能正是一个 API key，
/// 而错误消息会一路冒泡到日志与前端（T-01-26）。
pub fn validate_base_url(raw: &str) -> Result<Url, StoreError> {
    let url = Url::parse(raw.trim())
        .map_err(|e| StoreError::InvalidUrl(format!("could not be parsed as a URL: {e}")))?;

    if !ALLOWED_URL_SCHEMES.contains(&url.scheme()) {
        return Err(StoreError::InvalidUrl(format!(
            "scheme must be one of {ALLOWED_URL_SCHEMES:?}"
        )));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(StoreError::InvalidUrl("host must not be empty".into()));
    }
    // 密钥容器的边界建在**值**上，不只建在键名上：`llm.base_url` 这个键名再正常不过，
    // `is_secret_like_key` 对 `https://user:key@host/v1` 完全看不见，而这行值会随
    // sidecar 目录整体备份离开本机（docs/keychain-naming.md 不变量 1）。
    // 两个条件都要：`https://user@host/v1` 里 `password()` 是 `None`，只看 password 会漏。
    if !url.username().is_empty() || url.password().is_some() {
        return Err(StoreError::InvalidUrl(
            "must not carry credentials in the userinfo component".into(),
        ));
    }
    // 同一个洞的另一面：部分 OpenAI 兼容网关用 `?api-key=…` 形态传凭据。
    // 而 base_url 本身没有携带 query 或 fragment 的正当用途——真正的 query
    // 由 Phase 4 发请求时自己拼。
    if url.query().is_some() || url.fragment().is_some() {
        return Err(StoreError::InvalidUrl(
            "must not carry a query string or fragment".into(),
        ));
    }

    // 自建内网端点是合法场景，所以只警告不阻断；真正发出请求时在 Phase 4 复核。
    let is_loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if url.scheme() == "http" && !is_loopback {
        tracing::warn!(
            host = url.host_str().unwrap_or_default(),
            "LLM endpoint uses plaintext http to a non-loopback host"
        );
    }

    Ok(url)
}

/// 读一条配置。查不到是 `Ok(None)`，不是错误。
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, StoreError> {
    Ok(conn
        .query_row(SQL_GET, [key], |r| r.get::<_, String>(0))
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })?)
}

/// 写一条配置（同 key 覆盖）。
///
/// 两道守卫都长在这里而不是调用方：疑似密钥的**键名**一律拒绝，`base_url` 的**值**
/// 一律先过 [`validate_base_url`]（scheme / host / userinfo / query / fragment 四类）。
/// 放在调用方就等于「每个调用点都记得」，那是约定；放在这里它才是机制——
/// 绕过界面直接 invoke 也改变不了结果。
pub fn set_setting(tx: &Transaction, key: &str, value: &str) -> Result<(), StoreError> {
    if is_secret_like_key(key) {
        return Err(StoreError::InvalidSetting(format!(
            "key {key:?} looks like a secret; store it in the system keychain instead"
        )));
    }
    if key == SETTING_BASE_URL {
        validate_base_url(value)?;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    tx.execute(SQL_UPSERT, (key, value, now))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{open, Store, StoreError};

    fn fixture() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = open(&dir.path().join("t.db")).expect("open store");
        (dir, store)
    }

    fn count(store: &Store) -> i64 {
        store
            .read(|c| Ok(c.query_row("SELECT count(*) FROM settings", [], |r| r.get(0))?))
            .expect("count")
    }

    fn count_key(store: &Store, key: &str) -> i64 {
        store
            .read(|c| {
                Ok(c.query_row(
                    "SELECT count(*) FROM settings WHERE key = ?1",
                    [key],
                    |r| r.get(0),
                )?)
            })
            .expect("count key")
    }

    fn read(store: &Store, key: &str) -> Option<String> {
        store.read(|c| get_setting(c, key)).expect("get_setting")
    }

    #[test]
    fn settings_roundtrip() {
        let (_dir, store) = fixture();

        assert_eq!(read(&store, SETTING_MODEL), None, "未写入的 key 应读回 None");

        store
            .write(|tx| set_setting(tx, SETTING_MODEL, "first-model"))
            .expect("first write");
        assert_eq!(read(&store, SETTING_MODEL).as_deref(), Some("first-model"));

        store
            .write(|tx| set_setting(tx, SETTING_MODEL, "second-model"))
            .expect("overwrite");
        assert_eq!(read(&store, SETTING_MODEL).as_deref(), Some("second-model"));
        assert_eq!(count(&store), 1, "覆盖写不得产生第二行");

        let ts: i64 = store
            .read(|c| {
                Ok(c.query_row(
                    "SELECT updated_at FROM settings WHERE key = ?1",
                    [SETTING_MODEL],
                    |r| r.get(0),
                )?)
            })
            .expect("updated_at");
        assert!(ts > 1_700_000_000, "updated_at 应为当前 unix 秒，实得 {ts}");
    }

    #[test]
    fn settings_base_url_validation() {
        for ok in ["https://api.example.com/v1", "http://127.0.0.1:1234/v1"] {
            assert!(validate_base_url(ok).is_ok(), "{ok} 应被接受");
        }
        for bad in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "ftp://host/x",
            "",
            "   ",
            "not a url",
        ] {
            assert!(validate_base_url(bad).is_err(), "{bad:?} 应被拒绝");
        }

        // T-01-26：错误消息只描述规则，不回显传入的 value（它可能就是被误填的密钥）。
        let msg = validate_base_url("javascript:alert(1)")
            .unwrap_err()
            .to_string();
        assert!(!msg.contains("alert"), "错误消息不得回显 value: {msg}");

        let (_dir, store) = fixture();

        // 校验必须长在**写入路径**上，而不是指望调用方记得先调 validate_base_url。
        let rejected = store.write(|tx| set_setting(tx, SETTING_BASE_URL, "file:///etc/passwd"));
        assert!(
            matches!(rejected, Err(StoreError::InvalidUrl(_))),
            "非法 scheme 应返回 InvalidUrl，实得 {rejected:?}"
        );
        assert_eq!(count(&store), 0, "被拒的 base_url 不得进表");

        store
            .write(|tx| set_setting(tx, SETTING_BASE_URL, "https://api.example.com/v1"))
            .expect("valid base_url");
        assert_eq!(count(&store), 1);
    }

    /// 凭据藏在**值**里而不是键名里时，`is_secret_like_key` 完全看不见它:
    /// `llm.base_url` 这个键名再正常不过，而 `https://user:key@host/v1` 里的那串东西
    /// 会原样落进 `settings.value`，随 sidecar 目录整体备份离开本机
    /// （`docs/keychain-naming.md` 不变量 1 给出的正是这个理由）。
    ///
    /// 六组断言，每一条被删掉之后都会有一个具体的静默失败模式重新变得可能——
    /// 见每条旁边的注释。
    #[test]
    fn settings_base_url_rejects_credential_bearing_values() {
        // fixture 的密码位刻意不用 `sk-` 开头的长串（与 prism-llm 的 FIXTURE_SECRET 同一约定）：
        // 它对守卫的行使与真密钥无差别（`url.password()` 同样返回 `Some`），
        // 却不会让 check-secrets.sh 为了迁就 fixture 而放宽。
        const CREDENTIAL_USER: &str = "prism-test-user";
        const CREDENTIAL_PASS: &str = "prism-test-secret-value";
        const USER_AND_PASS: &str =
            "https://prism-test-user:prism-test-secret-value@api.vendor.com/v1";
        const USER_ONLY: &str = "https://prism-test-user@api.vendor.com/v1";
        const IN_QUERY: &str = "https://api.vendor.com/v1?api-key=prism-test-secret-value";
        const IN_FRAGMENT: &str = "https://api.vendor.com/v1#prism-test-secret-value";

        // ① 纯函数拒绝。删掉 userinfo 分支 → 前两条重新通过；删掉 query/fragment 分支
        //    → 后两条重新通过。两种放行都表现为「保存成功」，没有任何报错。
        //    只有用户名没有密码的形态（USER_ONLY）单独列出：`password()` 在那里是 `None`，
        //    只看 password 的守卫会漏掉它。
        for bad in [USER_AND_PASS, USER_ONLY, IN_QUERY, IN_FRAGMENT] {
            assert!(
                matches!(validate_base_url(bad), Err(StoreError::InvalidUrl(_))),
                "携带凭据的 base_url 应被拒绝: {bad:?} 实得 {:?}",
                validate_base_url(bad)
            );
        }

        let (_dir, store) = fixture();

        // ② 判别性所在：校验必须长在 `set_setting` 的**写入路径**上，而不是指望调用方
        //    先记得调 `validate_base_url`。这一条被删掉之后，「守卫是约定而非机制」
        //    就重新成为可能——绕过界面直接 invoke 即可写入。
        let rejected = store.write(|tx| set_setting(tx, SETTING_BASE_URL, USER_AND_PASS));
        assert!(
            matches!(rejected, Err(StoreError::InvalidUrl(_))),
            "带凭据的 base_url 在写入路径上应返回 InvalidUrl，实得 {rejected:?}"
        );

        // ③ 未留痕。缺了这一条，一个「先 INSERT 再报错」的实现也会绿——
        //    而备份带走的是表里的行，不是返回值。
        assert_eq!(count(&store), 0, "被拒的带凭据 base_url 不得进表");

        // ④ 不回显（T-01-26）。被误填进 base_url 的那串东西很可能就是密钥本身，
        //    而错误消息一路冒泡到本地日志与前端 DOM。
        let msg = validate_base_url(USER_AND_PASS).unwrap_err().to_string();
        assert!(
            !msg.contains(CREDENTIAL_USER),
            "错误消息不得回显用户名: {msg}"
        );
        assert!(
            !msg.contains(CREDENTIAL_PASS),
            "错误消息不得回显密码: {msg}"
        );

        // ⑤ 幂等。守卫无状态，第一次拒绝不会给第二次开门——
        //    缺了这一条，一个「只在首次写入时校验」的实现也会绿。
        let again = store.write(|tx| set_setting(tx, SETTING_BASE_URL, USER_AND_PASS));
        assert!(
            matches!(again, Err(StoreError::InvalidUrl(_))),
            "同一带凭据 URL 第二次写入仍应被拒，实得 {again:?}"
        );
        assert_eq!(count(&store), 0, "第二次被拒后 settings 表仍应为空");

        // ⑥ 阴性对照。守卫若写成「一律拒绝」，上面五条全都会绿——
        //    而那是一个把设置页彻底废掉的「修复」。
        assert!(
            validate_base_url("https://api.example.com/v1").is_ok(),
            "干净的 https 端点仍应被接受"
        );
        store
            .write(|tx| set_setting(tx, SETTING_BASE_URL, "https://api.example.com/v1"))
            .expect("干净的 base_url 应写入成功");
        assert_eq!(count(&store), 1, "干净的 base_url 应恰好写入一行");
    }

    #[test]
    fn settings_rejects_secret_like_keys() {
        let (_dir, store) = fixture();

        for key in [
            "llm.api_key",
            "LLM.API_KEY",
            "mcp.bearer_token",
            "some.Secret",
            "TOKEN",
        ] {
            assert!(is_secret_like_key(key), "{key} 应被判定为疑似密钥");
            let rejected = store.write(|tx| set_setting(tx, key, "placeholder-value"));
            assert!(
                matches!(rejected, Err(StoreError::InvalidSetting(_))),
                "{key} 的写入应被拒绝，实得 {rejected:?}"
            );
            assert_eq!(count_key(&store, key), 0, "{key} 不得出现在 settings 表中");
        }
        assert_eq!(count(&store), 0);

        // 阴性对照：守卫若写成「一律拒绝」，上面每一条也都会绿。
        assert!(!is_secret_like_key(SETTING_BASE_URL));
        assert!(!is_secret_like_key(SETTING_MODEL));
        store
            .write(|tx| set_setting(tx, SETTING_MODEL, "ordinary-value"))
            .expect("ordinary key should be accepted");
        assert_eq!(count(&store), 1);
    }
}
