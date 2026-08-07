pub mod cli;
pub use cli::*;

pub mod expanded_path;
pub use expanded_path::*;

#[cfg(feature = "async-io")]
pub mod file_open;
#[cfg(feature = "async-io")]
pub use file_open::*;

pub mod progress_bar;
pub use progress_bar::*;

// The target, not the `wasm` feature: a feature is additive, so one that *removes* a module leaves
// `lib::xdg`'s re-export — gated on the target since it was written — pointing at nothing the moment
// a workspace holding one wasm member unifies the feature onto a native build of this crate.
#[cfg(not(target_arch = "wasm32"))]
pub mod xdg;
