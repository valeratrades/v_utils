#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]

mod other;
pub use other::*;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "trades")]
pub mod trades;

#[cfg(feature = "macros")]
pub extern crate v_utils_macros as macros;

#[doc(hidden)]
pub mod __internal {
	pub use anyhow;
	pub use serde;
}

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "utils")]
pub mod utils;
