use v_utils_macros::SettingsNested;

// Unknown target inside `#[settings(skip(...))]` must be a hard error too — only
// `flag` and `env` are accepted.
#[derive(Clone, Debug, Default, SettingsNested)]
pub struct BadConfig {
	#[settings(skip(flagg))] //~ ERROR: unknown `flagg`
	pub nested: u32,
}

fn main() {}
