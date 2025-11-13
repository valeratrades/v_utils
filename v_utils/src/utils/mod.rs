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
	($fname:expr) => {
		color_eyre::install().unwrap();
		v_utils::utils::init_subscriber(v_utils::utils::LogDestination::xdg(env!("CARGO_PKG_NAME")).fname($fname));
	};
}

#[macro_export]
macro_rules! define_str_enum {
  ($(#[$meta:meta])* $vis:vis enum $name:ident {
    $($(#[$variant_meta:meta])* $variant:ident => $str:expr),* $(,)?
  }) => {
    $(#[$meta])*
    $vis enum $name {
      $($(#[$variant_meta])* $variant),*
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          $(Self::$variant => write!(f, "{}", $str)),*
        }
      }
    }

    impl std::str::FromStr for $name {
      type Err = eyre::Report;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $($str => Ok(Self::$variant)),*,
          _ => eyre::bail!("Invalid {} string: {}", stringify!($name).to_lowercase(), s),
        }
      }
    }
  };
}
