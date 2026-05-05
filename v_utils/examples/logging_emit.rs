//! Toy program exercising the `clientside!()` logging setup.
//!
//! Emits log events at all five levels in a 1:2:5:25:50 ratio
//! (error : warn : info : debug : trace). With the default `info+`
//! filter, debug/trace events are emitted but should not appear in
//! any of the captured outputs.
//!
//! Driven by `v_utils/tests/logging/file.rs`, which sets
//! `XDG_STATE_HOME` to a tempdir before spawning this binary.

use tracing::{debug, error, info, instrument, trace, warn};

fn main() {
	v_utils::clientside!();

	boot();
	compute();
	cleanup();
}

#[instrument]
fn boot() {
	info!("boot starting");
	debug!("loading config");
	for i in 0..5 {
		trace!(i, "init step");
	}
}

#[instrument]
fn compute() {
	info!("compute starting");
	info!("phase A");
	info!("phase B");
	warn!("compute saw a slow path");
	for i in 0..10 {
		debug!(i, "iteration");
	}
	for i in 0..30 {
		trace!(i, "fine-grained step");
	}
}

#[instrument]
fn cleanup() {
	info!("cleanup");
	warn!("late teardown");
	for i in 0..14 {
		debug!(i, "drain");
	}
	for i in 0..15 {
		trace!(i, "drain trace");
	}
	error!("simulated terminal error");
}
