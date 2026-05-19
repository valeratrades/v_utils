//! Verify that `#![feature(default_field_values)]` (RFC 3681) syntax compiles cleanly
//! through every v_utils derive macro that parses a user struct.
//!
//! `syn` 2.0 does not yet understand `pub field: T = expr,`, so each affected derive
//! macro pre-scrubs the `= expr` tail from its input. The built-in `#[derive(Default)]`
//! still sees the original tokens, so the actual default values keep working as the RFC
//! specifies.
//!
//! Per RFC 3681 default expressions must be const-evaluable — that's why this test uses
//! integer/bool literals and `None` rather than non-const expressions like `String::from`.
//!
//! `serde::Deserialize`'s own derive does NOT pre-scrub, so any field-default struct that
//! also needs `#[derive(Deserialize)]` would fail at serde's macro. The intended pattern is
//! `#[derive(MyConfigPrimitives)]`, which synthesises the `Deserialize` impl on the user's
//! behalf (via an internal helper) and is the entry point this crate's `Settings` /
//! `LiveSettings` story is built on.

#![feature(default_field_values)]

use v_utils_macros::{LiveSettings, MyConfigPrimitives, Settings};

#[derive(Clone, Debug, Default, LiveSettings, MyConfigPrimitives, Settings)]
#[allow(unused)]
struct AppConfig {
	#[serde(default)]
	pub port: u16 = 50736,
	#[serde(default)]
	pub debug: bool = true,
	#[serde(default)]
	pub workers: usize = 8,
	#[serde(default)]
	pub maybe_threads: Option<u32> = None,
}

#[test]
fn defaults_take_effect() {
	let c = AppConfig::default();
	assert_eq!(c.port, 50736);
	assert!(c.debug);
	assert_eq!(c.workers, 8);
	assert!(c.maybe_threads.is_none());
}

#[test]
fn try_build_signature_compiles() {
	// Proves `Settings` generated a working `try_build` on a struct with field defaults.
	let _f: fn(SettingsFlags) -> Result<AppConfig, v_utils::__internal::SettingsError> = AppConfig::try_build;
}
