use serde::{Deserialize, Serialize};
use v_utils_macros::Settings;

// `#[settings(flatten)]` resolves the flags struct through the `SettingsNested` trait.
// A flattened field whose type does not derive `SettingsNested` must produce a
// first-class trait-bound error naming the trait — not a missing-`__SettingsNested*`
// name-resolution error, and never a silent fallback. `Strategy` derives everything a
// nested settings field otherwise needs, so the *only* failure is the missing trait.
//
// The flags struct names `<Strategy as SettingsNested>::Flags` in many places (struct field,
// every derive, the config-source impl), so the diagnostic cascades; this directory runs with
// annotations off and pins the whole `.stderr` rather than tagging each line. The contract under
// test is just that the error is the named trait bound, not a `__SettingsNested*` resolution miss.
// `MyConfigPrimitives` already emits the serde impls, so `AppConfig` must NOT also derive them.
#[derive(Clone, Debug, Default, v_utils_macros::MyConfigPrimitives, Settings)]
pub struct AppConfig {
	#[settings(flatten)]
	#[serde(default)]
	pub strategy: Strategy,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Strategy {
	#[serde(default)]
	pub threshold: u32,
}

fn main() {}
