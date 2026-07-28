// RED 阶段：实现尚未写下，本文件只有测试。GREEN 阶段在此之上补齐实现。

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::collections::HashMap;

    /// 测试用的假密钥。刻意不用 `sk-` 开头的长串，避免与
    /// `scripts/check-secrets.sh` 的明文密钥正则撞车——那个扫描器是防线，
    /// 不该为了让 fixture 通过而放宽它。
    const FIXTURE_SECRET: &str = "prism-test-secret-value";
    const OTHER_FIXTURE_SECRET: &str = "prism-test-rotated-value";

    /// 装上进程级 mock store。
    ///
    /// 先 unset 再 set：前一个测试若在收尾前 panic，默认 store 会残留下来，
    /// 而 `set_default_store` 只是覆盖——先清一次让每个测试的起点确定。
    fn install_mock_store() {
        let _ = keyring_core::unset_default_store();
        keyring_core::set_default_store(
            keyring_core::mock::Store::new().expect("mock store is constructible"),
        );
    }

    /// 当前 (SERVICE, ACCOUNT_LLM_KEY) 下的条目数。
    fn llm_key_entry_count() -> usize {
        let spec = HashMap::from([("service", SERVICE), ("user", ACCOUNT_LLM_KEY)]);
        keyring_core::Entry::search(&spec)
            .expect("search the default store")
            .len()
    }

    #[test]
    #[serial]
    fn roundtrip_with_mock_store() {
        install_mock_store();

        assert!(get_api_key().expect("read empty store").is_none());

        set_api_key(FIXTURE_SECRET).expect("write the key");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(FIXTURE_SECRET)
        );

        delete_api_key().expect("delete the key");
        assert!(get_api_key().expect("read after delete").is_none());

        let _ = keyring_core::unset_default_store();
    }

    #[test]
    #[serial]
    fn no_key_is_not_an_error() {
        install_mock_store();

        // D-06：API key 可跳过，无 key 时应用照常启动。所以「没有条目」
        // 必须是 Ok(None) 而不是 Err——后者会让启动路径不得不吞错。
        let read = get_api_key();
        assert!(read.is_ok(), "missing key surfaced as an error");
        assert!(read.expect("checked above").is_none());

        let _ = keyring_core::unset_default_store();
    }

    #[test]
    #[serial]
    fn set_and_delete_are_idempotent() {
        install_mock_store();

        set_api_key(FIXTURE_SECRET).expect("first write");
        set_api_key(FIXTURE_SECRET).expect("second write of the same value");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(FIXTURE_SECRET),
            "repeated writes changed the stored value"
        );
        assert_eq!(
            llm_key_entry_count(),
            1,
            "repeated writes created a duplicate keychain entry"
        );

        // 覆写为另一个值同样只应留下一条条目（轮换 key 的路径）。
        set_api_key(OTHER_FIXTURE_SECRET).expect("rotate the key");
        assert_eq!(
            get_api_key().expect("read back").as_deref(),
            Some(OTHER_FIXTURE_SECRET)
        );
        assert_eq!(llm_key_entry_count(), 1, "rotation created a second entry");

        delete_api_key().expect("first delete");
        delete_api_key().expect("second delete must be a no-op, not an error");
        assert!(get_api_key().expect("read after delete").is_none());

        let _ = keyring_core::unset_default_store();
    }

    #[test]
    fn apikey_debug_is_redacted() {
        let key = ApiKey::new(FIXTURE_SECRET);
        let shown = format!("{key:?}");

        assert!(
            !shown.contains(FIXTURE_SECRET),
            "ApiKey leaked its secret through Debug: {shown}"
        );
        // 反向确认：`expose()` 才是取原文的唯一途径。
        assert_eq!(key.expose(), FIXTURE_SECRET);
    }

    #[test]
    fn keychain_errors_are_flattened_to_their_display_text() {
        // `keyring_core::Error` 的 derive Debug 会打印 BadEncoding 携带的原始字节。
        // LlmError 必须只保留 Display 文本，否则 `unwrap()` / `tracing` 的 `?err`
        // 就是一条把密钥打进日志的通道（T-01-04）。
        let raw = keyring_core::Error::BadEncoding(FIXTURE_SECRET.as_bytes().to_vec());
        let flattened = LlmError::from(raw);

        assert_eq!(
            format!("{flattened:?}"),
            r#"Keychain("Password data is not valid UTF-8")"#,
            "LlmError carried the raw keyring payload"
        );
        assert!(!format!("{flattened}").contains(FIXTURE_SECRET));
    }

    #[test]
    #[serial]
    #[ignore = "touches the real login keychain; run manually: cargo test -p prism-llm -- --ignored roundtrip_with_real_keychain"]
    fn roundtrip_with_real_keychain() {
        let _ = keyring_core::unset_default_store();
        init_default_store().expect("register the macOS keychain store");

        set_api_key(FIXTURE_SECRET).expect("write to the real keychain");
        assert_eq!(
            get_api_key().expect("read back from the real keychain").as_deref(),
            Some(FIXTURE_SECRET)
        );
        delete_api_key().expect("clean up the real keychain entry");
        assert!(get_api_key().expect("read after delete").is_none());

        let _ = keyring_core::unset_default_store();
    }
}
