use v_utils_macros::SettingsNested;

// Unknown struct-level `#[settings(...)]` ident must be a hard error. A typo here
// (e.g. `prefxi` instead of `prefix`) would otherwise silently fall back to the
// default prefix, producing wrong flag names and config paths with no diagnostic.
#[derive(Clone, Debug, Default, SettingsNested)]
#[settings(prefxi = "strategy")] //~ ERROR: unknown `prefxi`
pub struct BadConfig {
	pub nested: u32,
}

fn main() {}
