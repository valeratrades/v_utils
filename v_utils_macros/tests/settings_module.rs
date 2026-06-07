//! `write_module()` emits a NixOS-style options module from the struct's JSON Schema.
//!
//! Two layers, per the boundaries we actually care about:
//! - the emitted module *text* declares the right option types (pure, fast);
//! - a config that `import`s the module type-checks under `lib.evalModules`, and a wrongly-typed
//!   value is *rejected* by Nix evaluation (the whole point — eval-time type awareness). That
//!   layer is gated on `nix` + `<nixpkgs>` being available, so it self-skips in a nix-less env.
//!
//! (The graceful-degradation case — `write_module()` erroring without `#[derive(JsonSchema)]` —
//! is covered by `settings_schema_optional.rs`'s sibling `write_schema` assertion; both share the
//! same `GetSchema` autoref gate, so one negative test suffices for the gate itself.)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use v_utils_macros::{Settings, SettingsNested};

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize, SettingsNested)]
#[allow(unused)]
struct Logging {
	#[serde(default)]
	level: String,
	#[serde(default)]
	file: Option<String>,
	#[serde(default)]
	tags: Vec<String>,
}

#[derive(Clone, Debug, Default, JsonSchema, v_utils_macros::MyConfigPrimitives, Settings)]
#[allow(unused)]
struct ModuleConfig {
	#[serde(default)]
	host: String,
	#[serde(default)]
	port: u16,
	#[settings(flatten)]
	#[serde(default)]
	logging: Logging,
}

/// Write the module into a throwaway `$XDG_CONFIG_HOME` and return (its path, its contents).
fn emit_module() -> (std::path::PathBuf, String) {
	let tmp = tempfile::tempdir().unwrap();
	// `write_module` (non-xdg branch) resolves the config dir via `xdg_config_fallback`,
	// which honors `$XDG_CONFIG_HOME`. SAFETY: single-threaded test.
	unsafe {
		std::env::set_var("XDG_CONFIG_HOME", tmp.path());
	}
	let path = ModuleConfig::write_module().expect("JsonSchema is derived, so this must succeed");
	let contents = std::fs::read_to_string(&path).unwrap();
	// Keep the tempdir alive by leaking it — the process is about to exit anyway, and we need the
	// file to outlive this fn for the eval layer below.
	std::mem::forget(tmp);
	(path, contents)
}

#[test]
fn emits_expected_option_types() {
	let (path, m) = emit_module();
	assert_eq!(path.extension().and_then(|e| e.to_str()), Some("nix"));

	assert!(m.contains("{ lib, ... }:"), "module must be a lib-taking function:\n{m}");
	assert!(m.contains("options ="), "module must declare an options set:\n{m}");
	assert!(m.contains("host = lib.mkOption { type = lib.types.str;"), "host should be a str option:\n{m}");
	assert!(m.contains("port = lib.mkOption { type = lib.types.int;"), "port (u16) should map to types.int:\n{m}");
	// Flattened nested struct -> submodule with its own options.
	assert!(m.contains("logging = lib.mkOption { type = lib.types.submodule"), "logging should be a submodule:\n{m}");
	// Option<String> -> nullOr str, with default null so it may be omitted.
	assert!(m.contains("lib.types.nullOr lib.types.str"), "optional file should be nullOr str:\n{m}");
	assert!(m.contains("default = null;"), "optional field should carry `default = null;`:\n{m}");
	// Vec<String> -> listOf str.
	assert!(m.contains("lib.types.listOf lib.types.str"), "tags should be listOf str:\n{m}");
}

/// `true` if `nix` and a `<nixpkgs>` channel are usable; the eval-layer test no-ops otherwise.
fn nix_available() -> bool {
	std::process::Command::new("nix")
		.args(["eval", "--impure", "--expr", "(import <nixpkgs> {}).lib.types.int.name"])
		.output()
		.map(|o| o.status.success())
		.unwrap_or(false)
}

/// Evaluate `<module> + <config>` through `lib.evalModules` and report whether it type-checks.
/// `config_body` is the Nix attrset body the user would write (without the surrounding braces).
fn eval_check(module_path: &std::path::Path, config_body: &str) -> bool {
	let expr = format!(
		r#"let pkgs = import <nixpkgs> {{}}; cfg = (pkgs.lib.evalModules {{ modules = [ {module} ({{ ... }}: {{ {body} }}) ]; }}).config; in builtins.deepSeq cfg true"#,
		module = module_path.display(),
		body = config_body,
	);
	std::process::Command::new("nix")
		.args(["eval", "--impure", "--expr", &expr])
		.output()
		.map(|o| o.status.success())
		.unwrap_or(false)
}

#[test]
fn nix_evalmodules_typechecks_config() {
	if !nix_available() {
		eprintln!("skipping nix evalModules test: nix/<nixpkgs> unavailable");
		return;
	}
	let (path, _) = emit_module();

	// A well-typed config must evaluate.
	assert!(
		eval_check(&path, r#"host = "localhost"; port = 8080; logging = { level = "info"; tags = [ "a" ]; };"#),
		"a correctly-typed config should pass evalModules"
	);

	// A wrongly-typed value (string where int expected) must be rejected by Nix.
	assert!(
		!eval_check(&path, r#"host = "localhost"; port = "not-an-int"; logging = { level = "info"; tags = [ ]; };"#),
		"a string `port` should fail evalModules type checking"
	);
}
