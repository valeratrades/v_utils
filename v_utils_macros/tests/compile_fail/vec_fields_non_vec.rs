use v_utils_macros::VecFieldsFromVecStr;

// All fields must be Vec<T>.
#[derive(Debug, VecFieldsFromVecStr)] //~ ERROR: proc-macro derive panicked
pub struct WrongFieldType {
	value: f64,
}
