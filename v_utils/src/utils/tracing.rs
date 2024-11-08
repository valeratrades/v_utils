use std::{
	fs::File,
	io::Write,
	path::Path,
	sync::{atomic::Ordering, Arc},
	time::Duration,
};

use eyre::{bail, eyre, Report, Result, WrapErr};
use serde::{de::DeserializeOwned, Deserializer};
use tokio::{runtime::Runtime, time::sleep};
use tracing::{error, info, instrument, subscriber::set_global_default, warn, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer, Type};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{
	fmt::{self, MakeWriter},
	layer::SubscriberExt as _,
	prelude::*,
	util::SubscriberInitExt as _,
	EnvFilter, Registry,
};

/// # Panics
/// Set "TEST_LOG=1" to redirect to stdout
pub fn init_subscriber(log_path: Option<Box<Path>>) {
	let setup = |make_writer: Box<dyn Fn() -> Box<dyn Write> + Send + Sync>| {
		//let tokio_console_artifacts_filter = EnvFilter::new("tokio[trace]=off,runtime[trace]=off");
		let formatting_layer = tracing_subscriber::fmt::layer().json().pretty().with_writer(make_writer).with_file(true).with_line_number(true)/*.with_filter(tokio_console_artifacts_filter)*/;

		let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(tracing_subscriber::EnvFilter::new("info"));
		//let env_filter = env_filter
		//      .add_directive("tokio=off".parse().unwrap())
		//      .add_directive("runtime=off".parse().unwrap());

		let error_layer = ErrorLayer::default();

		let console_layer = console_subscriber::spawn::<Registry>(); // does nothing unless `RUST_LOG=tokio=trace,runtime=trace`. But how do I make it not write to file for them?

		tracing_subscriber::registry()
			.with(console_layer)
			.with(env_filter)
			.with(formatting_layer)
			.with(error_layer)
			.init();
		//tracing_subscriber::registry()
		//  .with(tracing_subscriber::layer::Layer::and_then(formatting_layer, error_layer).with_filter(env_filter))
		//  .with(console_layer)
		//  .init();
	};

	match log_path {
		Some(path) => {
			let path = path.to_owned();

			// Truncate the file before setting up the logger
			{
				let _ = std::fs::OpenOptions::new()
					.create(true)
					.write(true)
					.truncate(true)
					.open(&path)
					.expect("Failed to truncate log file");
			}

			setup(Box::new(move || {
				let file = std::fs::OpenOptions::new().create(true).append(true).open(&path).expect("Failed to open log file");
				Box::new(file) as Box<dyn Write>
			}));
		}
		None => {
			setup(Box::new(|| Box::new(std::io::stdout())));
		}
	};

	trace_the_init();
}

use std::{
	collections::BTreeMap,
	env::{args_os, current_dir, current_exe, vars_os},
};
fn trace_the_init() {
	let args: Vec<_> = args_os().collect();
	let vars: BTreeMap<_, _> = vars_os().collect();
	info!("Executed as {exe:?} in {dir:?}\n", exe = current_exe(), dir = current_dir(),);
	info!("Arguments: {args:#?}\n", args = args);
	info!("Environment: {vars:#?}\n", vars = vars);
}
