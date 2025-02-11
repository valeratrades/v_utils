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

#[cfg(feature = "tracing")]
#[cfg(not(feature = "wasm"))]
#[macro_export]
macro_rules! clientside {
	() => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME")));
	};
}

//HACK: all this code-duplication for one line add
#[cfg(feature = "tracing")]
#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! clientside {
	() => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME")));

		#[cfg(target_arch = "wasm32")]
		v_utils::__internal::console_error_panic_hook::set_once(); // for wasm32 targets exclusively.
		#[cfg(target_arch = "wasm32")]
		_ = v_utils::__internal::console_log::init_with_level(log::Level::Debug);
	};
}
