//! 非密钥配置的 k/v 读写（D-05）。

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
