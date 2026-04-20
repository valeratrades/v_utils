use v_utils_macros::wrap_err;

// #[own] requires a tuple variant with exactly one field.
#[wrap_err] //~ ERROR: custom attribute panicked
#[derive(Debug)]
pub enum MyError {
	#[own]
	Named { field: String },
}
