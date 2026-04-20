use v_utils_macros::TryParseVariants;

// TryParseVariants only works on enums.
#[derive(Debug, TryParseVariants)] //~ ERROR: proc-macro derive panicked
pub struct NotAnEnum {
	value: f64,
}
