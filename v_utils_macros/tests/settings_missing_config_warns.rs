//! When no config file exists on disk, `Settings::try_build` must still succeed
//! by falling back to env + flags + field defaults — but it must NOT do so
//! silently. It emits `warning: no config file found ...` to stderr.
//!
//! This test pins the *contract* (missing file ⇒ Ok-from-defaults, never a
//! silent error) which is the regression that actually bit a downstream daemon:
//! a mislocated config degraded to defaults with no signal. The stderr warning
//! itself is a side effect — eyeball it with `--nocapture` (capturing the test
//! process's own stderr in-process is brittle and not worth the machinery).

use clap::Parser;
use v_utils_macros::Settings;

#[derive(Debug, Parser)]
#[allow(unused)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

/// Every field has a default, so a build with no config file present is fully
/// determined — the only question the test asks is whether `try_build` reaches
/// it (warn-and-build) rather than bailing (silent-or-loud failure).
#[derive(Clone, Debug, Default, v_utils_macros::MyConfigPrimitives, Settings)]
struct DefaultedConfig {
	#[serde(default)]
	host: String,
	#[serde(default)]
	port: u16,
	#[serde(default)]
	debug: bool,
}

#[test]
fn missing_config_builds_from_defaults() {
	// `config: None` + app name `v_utils_macros` (CARGO_PKG_NAME) ⇒ the macro
	// searches `~/.config/v_utils_macros.{nix,toml,…}`, finds nothing in a clean
	// environment, and takes the zero-files branch. No env vars are set for the
	// `V_UTILS_MACROS__*` prefix, so the result is pure field defaults.
	assert!(
		std::env::var_os("V_UTILS_MACROS__HOST").is_none(),
		"test env is polluted with V_UTILS_MACROS__HOST — the defaults assertion below would be meaningless"
	);
	let flags = SettingsFlags {
		config: None,
		yes: false,
		host: None,
		port: None,
		debug: None,
	};

	let cfg = DefaultedConfig::try_build(flags).expect("missing config file must build from defaults, not error");

	assert_eq!(cfg.host, "");
	assert_eq!(cfg.port, 0);
	assert!(!cfg.debug);
}
