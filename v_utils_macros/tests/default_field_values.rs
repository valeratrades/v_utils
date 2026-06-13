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
//! `LiveSettings` story is built on. `MyConfigPrimitives` also threads the stripped
//! `= expr` defaults into the synthesised Helper as `#[serde(default = "...")]`, so
//! missing fields deserialize to the RFC-3681 default rather than erroring.

#![feature(default_field_values)]

use v_utils_macros::{ConfigJsonSchema, LiveSettings, MyConfigPrimitives, Settings};

// `ConfigJsonSchema` stands in for `schemars::JsonSchema`: the latter's derive can't parse the
// `field: T = expr` tails, so before this macro existed a field-default struct could not also
// produce a JSON Schema (the `Settings` macro's schema/module export silently degraded to `None`).
#[derive(Clone, ConfigJsonSchema, Debug, Default, LiveSettings, MyConfigPrimitives, Settings)]
#[allow(unused)]
struct AppConfig {
	pub port: u16 = 50736,
	pub debug: bool = true,
	pub workers: usize = 8,
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
fn deserialize_uses_field_defaults() {
	// Empty TOML — every field is missing. Without the macro propagating
	// `= expr` defaults into serde, this would error with "missing field …".
	let c: AppConfig = toml::from_str("").expect("empty config should deserialize via field defaults");
	assert_eq!(c.port, 50736);
	assert!(c.debug);
	assert_eq!(c.workers, 8);
	assert!(c.maybe_threads.is_none());

	// Partial TOML — present fields override, absent fields fall back.
	let c: AppConfig = toml::from_str("port = 9000\n").expect("partial config should deserialize");
	assert_eq!(c.port, 9000);
	assert!(c.debug);
	assert_eq!(c.workers, 8);
}

#[test]
fn try_build_signature_compiles() {
	// Proves `Settings` generated a working `try_build` on a struct with field defaults.
	let _f: fn(SettingsFlags) -> Result<AppConfig, v_utils::__internal::SettingsError> = AppConfig::try_build;
}

#[test]
fn config_json_schema_emits_named_schema() {
	// `ConfigJsonSchema` must produce a real schema (not degrade), titled after the user struct
	// rather than the internal mirror, with every field present.
	let schema = v_utils::__internal::schemars::schema_for!(AppConfig);
	let json = v_utils::__internal::serde_json::to_value(&schema).unwrap();
	assert_eq!(json["title"], "AppConfig", "schema title must be the real struct name, not the mirror");
	let props = &json["properties"];
	for field in ["port", "debug", "workers", "maybe_threads"] {
		assert!(props.get(field).is_some(), "schema missing field `{field}`: {json}");
	}
}
