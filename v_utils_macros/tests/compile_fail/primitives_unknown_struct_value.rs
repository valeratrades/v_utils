use v_utils_macros::MyConfigPrimitives;

// Unknown content inside a struct-level `#[primitives(...)]` must be a hard error.
// A typo here would otherwise silently fail to opt out of the generated Serialize impl.
#[derive(Clone, Debug, MyConfigPrimitives)]
#[primitives(skip_serialise)] //~ ERROR: unknown `#[primitives(skip_serialise)]` on struct
pub struct BadConfig {
	pub raw: String,
}

fn main() {}
