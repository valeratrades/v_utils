use v_utils_macros::ScreamIt;

// ScreamIt only works on enums.
#[derive(Debug, ScreamIt)] //~ ERROR: proc-macro derive panicked
pub struct NotAnEnum {
	value: String,
}
