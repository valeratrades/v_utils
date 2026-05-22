#![feature(default_field_values)]

use v_utils_macros::MyConfigPrimitives;

// Same hazard as `private_value_with_smart_default.rs`, but via the nightly
// `field: T = expr` syntax instead of the SmartDefault attribute.
#[derive(Clone, Debug, Default, MyConfigPrimitives)] //~ ERROR: proc-macro derive panicked
pub struct BadConfig {
	#[private_value]
	pub port: u16 = 42,
}
