//! 文件系统监视层：notify + debouncer 的落点。
//!
//! Phase 1 plan 01（D-08）只建立骨架与真实依赖声明——REQ-1.4.3 的合并语义、
//! `.prismdocs/` 忽略规则与 10s 呈现预算是 Phase 2 的内容。
//! 本文件的函数存在的意义是让 notify / notify-debouncer-full **真正被编译进依赖树**，
//! 从而使 `cargo tree -d` 的版本冲突检查在 Phase 1 就覆盖它们。

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("filesystem watch error: {0}")]
    Watch(#[from] notify::Error),
}

/// 当前平台上 `notify` 选定的 watcher 后端类型名（macOS 上为 FSEvents 实现）。
pub fn watcher_backend_name() -> &'static str {
    std::any::type_name::<notify::RecommendedWatcher>()
}

/// debouncer 使用的缓存后端类型名——让 notify-debouncer-full 进入真实依赖面。
pub fn debounce_cache_name() -> &'static str {
    std::any::type_name::<notify_debouncer_full::RecommendedCache>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watcher_backend_comes_from_notify() {
        let name = watcher_backend_name();
        assert!(
            name.starts_with("notify::"),
            "unexpected watcher backend: {name}"
        );
    }

    #[test]
    fn debounce_cache_comes_from_the_debouncer_crate() {
        let name = debounce_cache_name();
        assert!(
            name.starts_with("notify_debouncer_full::"),
            "unexpected debounce cache: {name}"
        );
    }

    /// 依赖真的可用（不只是声明）：在本机构造一次平台 watcher。
    #[test]
    fn a_recommended_watcher_can_actually_be_constructed() {
        let watcher = notify::recommended_watcher(|_res: notify::Result<notify::Event>| {})
            .map_err(FsError::from)
            .expect("construct the platform watcher");
        drop(watcher);
    }
}
