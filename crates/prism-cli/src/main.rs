//! `prismdocs-helper`——供编码 agent 侧使用的轻量 CLI helper（D-07 / D-10）。
//!
//! **D-10：本 binary 不得依赖任何 `prism-*` engine crate。** 它未来要作为
//! Tauri `externalBin` 单独签名公证，依赖面必须小且可独立审计；
//! 该性质由 `cargo tree -p prism-cli` 中不出现任何 `prism-` crate 断言。
//!
//! Phase 1 plan 01 只建立占位形态：`help` 打印用法并退出 0，`doctor` 做一次
//! 不发网络请求的本机自检。真正的 `headers` 与 `check-feedback` 子命令在 Phase 6。

use std::process::ExitCode;

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
prismdocs-helper — PrismDocs MCP helper

USAGE:
    prismdocs-helper <SUBCOMMAND>

SUBCOMMANDS:
    help        Print this message (default)
    doctor      Check that the local HTTP and Keychain backends initialise

NOT YET IMPLEMENTED (Phase 6):
    headers          Emit the loopback MCP bearer header for a coding agent
    check-feedback   SessionStart hook: report pending comment feedback
";

#[derive(Debug, thiserror::Error)]
enum HelperError {
    #[error("http backend unavailable: {0}")]
    Http(#[from] reqwest::Error),
    #[error("keychain unavailable: {0}")]
    Keychain(#[from] keyring_core::Error),
    #[error("unknown subcommand `{0}`\n\n{USAGE}")]
    UnknownSubcommand(String),
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("prismdocs-helper: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, HelperError> {
    match args.first().map(String::as_str) {
        None | Some("help") | Some("--help") | Some("-h") => Ok(USAGE.to_string()),
        Some("doctor") => doctor(),
        Some(other) => Err(HelperError::UnknownSubcommand(other.to_string())),
    }
}

fn user_agent() -> String {
    format!("PrismDocs-helper/{CRATE_VERSION}")
}

/// 本机自检：只构造客户端与解析后端类型，**不发任何网络请求、不读取任何密钥**。
fn doctor() -> Result<String, HelperError> {
    let client = reqwest::Client::builder()
        .user_agent(user_agent())
        .build()?;
    drop(client);

    let keychain = std::any::type_name::<apple_native_keyring_store::keychain::Store>();
    Ok(format!(
        "prismdocs-helper {CRATE_VERSION}\n\
         http backend:     ok ({})\n\
         keychain backend: {keychain}\n",
        user_agent()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_arguments_prints_usage() {
        let out = run(&[]).expect("usage");
        assert!(out.starts_with("prismdocs-helper —"), "unexpected: {out}");
    }

    #[test]
    fn help_prints_usage() {
        assert_eq!(run(&argv(&["help"])).unwrap(), USAGE);
        assert_eq!(run(&argv(&["--help"])).unwrap(), USAGE);
    }

    #[test]
    fn an_unknown_subcommand_is_an_error_that_still_shows_usage() {
        let err = run(&argv(&["nope"])).expect_err("should reject");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown subcommand `nope`"),
            "unexpected: {msg}"
        );
        assert!(msg.contains("SUBCOMMANDS:"), "unexpected: {msg}");
    }

    #[test]
    fn doctor_reports_both_backends_without_touching_the_network() {
        let out = run(&argv(&["doctor"])).expect("doctor");
        assert!(out.contains("http backend:     ok"), "unexpected: {out}");
        assert!(
            out.contains("apple_native_keyring_store::keychain::"),
            "unexpected: {out}"
        );
    }
}
