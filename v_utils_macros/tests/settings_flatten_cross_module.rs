//! Regression for the bug this trait-based wiring fixes: a `#[settings(flatten)]` child whose
//! type lives in a *different* module and is imported by name alone. Before `SettingsNested`
//! became a trait, the parent macro fabricated a bare `__SettingsNestedArbStrategyConfig` ident
//! that only resolved if the companion struct happened to be in scope — so importing only the
//! type (`use inner::Sub;`) failed with `cannot find type __SettingsNested...`. This file
//! exercises exactly that shape; if it compiles, name resolution now follows the type.

use clap::Parser;
use v_utils_macros::Settings;

mod inner {
	use serde::{Deserialize, Serialize};
	use v_utils_macros::SettingsNested;

	#[derive(Clone, Debug, Default, Deserialize, Serialize, SettingsNested)]
	pub struct Sub {
		#[serde(default)]
		pub threshold: u32,
		#[serde(default)]
		pub label: String,
	}
}

// By name only — the companion `__SettingsNestedSub` is deliberately NOT imported.
use inner::Sub;

#[derive(Debug, Parser)]
#[allow(unused)]
struct Cli {
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}
#[derive(Clone, Debug, Default, v_utils_macros::MyConfigPrimitives, Settings)]
#[allow(unused)]
struct AppConfig {
	#[serde(default)]
	host: String,
	#[settings(flatten)]
	#[serde(default)]
	sub: Sub,
}

#[test]
fn flatten_child_in_other_module_resolves_by_name() {
	use v_utils::__internal::config::Source as _;

	// Build the flags the way they're actually built: clap parsing the flattened child's
	// `--sub-*` args (its fields are a private impl detail, never named by hand). The flattened
	// child type `Sub` was imported by name only — the old `__SettingsNested*` name fabrication
	// would have failed to compile this whole struct.
	let cli = Cli::parse_from(["app", "--host", "localhost", "--sub-threshold", "7", "--sub-label", "hi"]);

	// `collect` walks the flattened child via `<Sub as SettingsNested>::collect_config`, proving
	// the trait method replaced the old inherent one and that paths resolve cross-module.
	let map = cli.settings_flags.collect().expect("flag source collects");
	assert_eq!(map.get("sub.threshold").map(ToString::to_string), Some("7".to_string()));
	assert_eq!(map.get("sub.label").map(ToString::to_string), Some("hi".to_string()));
}
