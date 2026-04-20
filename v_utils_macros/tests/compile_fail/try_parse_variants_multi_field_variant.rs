use v_utils_macros::TryParseVariants;

// Every variant must be a single-field tuple variant — multi-field variants are not supported.
#[derive(Debug, TryParseVariants)] //~ ERROR: proc-macro derive panicked
pub enum Strategy {
	Trailing(f64, f64),
}
