#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
#![feature(stmt_expr_attributes)]

mod other;
pub use other::*;

mod prelude;
#[cfg(feature = "lite")]
pub use prelude::{clientside as prelude_clientside, libside as prelude_libside};
// of course it's included unconditionally - the crate itself is called "v_utils"
pub mod utils;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "trades")]
pub mod trades;

#[cfg(feature = "macros")]
pub extern crate v_utils_macros as macros;

#[doc(hidden)]
pub mod __internal {
	pub extern crate eyre;
	pub extern crate serde;
}

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "distributions")]
pub mod distributions;
