use v_utils_macros::MyConfigPrimitives;

// `#[private_value]` resolves from `String` / `{ env = "..." }` at deserialization,
// so a typed `#[default(expr)]` would mismatch the synthesized Helper field type.
// The macro must reject this combo at expansion time.
#[derive(Clone, Debug, Default, MyConfigPrimitives)] //~ ERROR: proc-macro derive panicked
pub struct BadConfig {
	#[private_value]
	#[default(42u16)] //~ ERROR: the `#[default]` attribute may only be used on unit enum variants
	pub port: u16,
}
