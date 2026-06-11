use v_utils_macros::MyConfigPrimitives;

// Unknown content inside a field-level `#[primitives(...)]` must be a hard error,
// not silently ignored (which would leave the field unwrapped with no diagnostic).
#[derive(Clone, Debug, MyConfigPrimitives)]
pub struct BadConfig {
	#[primitives(skp)] //~ ERROR: unknown `#[primitives(skp)]` on field
	pub raw: String,
}

fn main() {}
