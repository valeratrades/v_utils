use v_utils_macros::OptionalFieldsFromVecStr;

// All fields must be Option<T>.
#[derive(Debug, OptionalFieldsFromVecStr)] //~ ERROR: proc-macro derive panicked
pub struct WrongFieldType {
	value: f64,
}
