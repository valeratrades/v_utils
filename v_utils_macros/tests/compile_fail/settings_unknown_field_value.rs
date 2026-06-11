use v_utils_macros::SettingsNested;

// Unknown content inside a field-level `#[settings(...)]` must be a hard error,
// not silently ignored (which would drop the directive with no diagnostic).
#[derive(Clone, Debug, Default, SettingsNested)]
pub struct BadConfig {
	#[settings(flaten)] //~ ERROR: unknown `flaten`
	pub nested: u32,
}

fn main() {}
