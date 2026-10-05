use v_utils_macros::SettingsNested;

// A non-`Option` field is required in every profile; `required_in` on it is a contradiction.
#[derive(Clone, Debug, Default, SettingsNested)]
pub struct BadConfig {
	#[settings(required_in("production"))]
	pub key: String, //~ ERROR: `required_in` is for `Option` fields
}

fn main() {}
