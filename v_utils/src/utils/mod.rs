pub mod eyre;
pub use eyre::*;

pub mod snapshots;
pub use snapshots::*;

pub mod format;
pub use format::*;

pub mod serde;
pub use serde::*;

#[cfg(feature = "tracing")]
pub mod tracing;
#[cfg(feature = "tracing")]
pub use tracing::*;

/// # HACK
/// Assumes that `color_eyre` is in scope
#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! clientside {
	() => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME")));
	};
}
