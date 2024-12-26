use std::{
	io::Write,
	path::{Path, PathBuf},
};

use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt as _, prelude::*, Registry};

#[derive(Clone, Debug, Default)]
pub enum LogDestination {
	#[default]
	Stdout,
	File(Box<Path>),
	XdgDataHome(String),
}
impl LogDestination {
	/// Helper for creating [XdgDataHome](LogDestination::XdgDataHome) variant
	pub fn xdg_data_home<S: Into<String>>(name: S) -> Self {
		LogDestination::XdgDataHome(name.into())
	}
}

/// # Panics (iff ` Some(path)` && `path`'s parent dir doesn't exist || `path` is not writable)
/// Set "TEST_LOG=1" to redirect to stdout
pub fn init_subscriber(log_destination: LogDestination) {
	let mut logs_during_init: Vec<Box<dyn FnOnce()>> = Vec::new();
	let mut setup = |make_writer: Box<dyn Fn() -> Box<dyn Write> + Send + Sync>| {
		//let tokio_console_artifacts_filter = EnvFilter::new("tokio[trace]=off,runtime[trace]=off");
		let formatting_layer = tracing_subscriber::fmt::layer().json().pretty().with_writer(make_writer).with_file(true).with_line_number(true)/*.with_filter(tokio_console_artifacts_filter)*/;

		let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or({
			logs_during_init.push(Box::new(|| {
				tracing::warn!("Couldn't construct a `tracing_subscriber::EnvFilter` instance from environment, defaulting to info level logging")
			}));
			tracing_subscriber::EnvFilter::new("debug")
		});
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

	fn destination_is_path<F, P>(path: P, setup: F)
	where
		P: Into<PathBuf> + Sized,
		//F: FnOnce() -> Box<dyn Write> + 'static, {
		F: FnOnce(Box<dyn Fn() -> Box<dyn Write> + Send + Sync>) -> (), {
		let path = path.into();

		// Truncate the file before setting up the logger
		{
			let _ = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&path).expect(&format!(
				"Couldn't open {} for writing. If its parent directory doesn't exist, create it manually first",
				path.display(),
			));
		}

		setup(Box::new(move || {
			let file = std::fs::OpenOptions::new().create(true).append(true).open(&path).expect("Failed to open log file");
			Box::new(file) as Box<dyn Write>
		}));
	}

	match log_destination {
		LogDestination::File(path) => {
			destination_is_path(path, setup);
		}
		LogDestination::Stdout => {
			setup(Box::new(|| Box::new(std::io::stdout())));
		}
		LogDestination::XdgDataHome(name) => {
			let associated_data_home = xdg::BaseDirectories::with_prefix(name).unwrap().create_data_directory("").unwrap();
			let log_path = associated_data_home.join(".log");
			destination_is_path(log_path, setup);
		}
	};

	for log in logs_during_init {
		log();
	}

	trace_the_init(); //? Should I make this a trace?
}

use std::{
	collections::BTreeMap,
	env::{args_os, current_dir, current_exe, vars_os},
};
fn trace_the_init() {
	let args: Vec<_> = args_os().collect();
	let vars: BTreeMap<_, _> = vars_os().collect();
	tracing::trace!("Executed as {exe:?} in {dir:?}\n", exe = current_exe(), dir = current_dir(),);
	tracing::trace!("Arguments: {args:#?}\n", args = args);
	tracing::trace!("Environment: {vars:#?}\n", vars = vars);
}
