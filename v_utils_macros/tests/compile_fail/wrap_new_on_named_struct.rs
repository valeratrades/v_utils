use v_utils_macros::WrapNew;

// WrapNew requires a single-field tuple struct.
#[derive(Debug, WrapNew)] //~ ERROR: proc-macro derive panicked
pub struct Named {
	inner: String,
}
