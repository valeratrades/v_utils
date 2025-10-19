use std::{borrow::Cow, io::Write, path::PathBuf};

use tracing::{info, warn};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, prelude::*};

/// # Panics (iff ` Some(path)` && `path`'s parent dir doesn't exist || `path` is not writable)
/// Set "TEST_LOG=1" to redirect to stdout
pub fn init_subscriber(log_destination: LogDestination) {
	let mut logs_during_init: Vec<Box<dyn FnOnce()>> = Vec::new();
	let mut setup = |make_writer: Box<dyn Fn() -> Box<dyn Write> + Send + Sync>| {
		//TODO: 	console_error_panic_hook::set_once(); // for wasm32 targets exclusively.
		//let tokio_console_artifacts_filter = EnvFilter::new("tokio[trace]=off,runtime[trace]=off");
		//TEST: if `with_ansi(false)` removes the need for `AnsiEsc` completely
		let formatting_layer = tracing_subscriber::fmt::layer().json().pretty().with_writer(make_writer).with_ansi(false).with_file(true).with_line_number(true)/*.with_filter(tokio_console_artifacts_filter)*/;

		let env_filter = filter_with_directives(&mut logs_during_init);

		let error_layer = ErrorLayer::default();

		// freaks out if it's built into a binary, and then two instances of it are created.
		//TODO: figure out how to limit this to debug builds \
		//#[feature("tokio_full")]
		//let console_layer = console_subscriber::spawn::<Registry>(); // does nothing unless `RUST_LOG=tokio=trace,runtime=trace`. But how do I make it not write to file for them?
		//
		//TODO!!!: check out [tracing appender](https://docs.rs/tracing-appender/latest/tracing_appender/) - seems very useful for long-running processes. Probably should add it here + config for it in the same place as directives conf
		tracing_subscriber::registry()
			//.with(console_layer)
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
		F: FnOnce(Box<dyn Fn() -> Box<dyn Write> + Send + Sync>), {
		let path = path.into();

		// Truncate the file before setting up the logger
		{
			let _ = std::fs::OpenOptions::new()
				.create(true)
				.write(true)
				.truncate(true)
				.open(&path)
				.unwrap_or_else(|_| panic!("Couldn't open {} for writing. If its parent directory doesn't exist, create it manually first", path.display()));
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
		#[cfg(not(target_arch = "wasm32"))]
		LogDestination::Xdg(name) => {
			let associated_state_home = xdg::BaseDirectories::with_prefix(name).create_state_directory("").unwrap();
			let log_path = associated_state_home.join(".log");
			destination_is_path(log_path, setup);
		}
	};

	for log in logs_during_init {
		log();
	}
	info!("Starting ...");

	trace_the_init(); //? Should I make this a trace?
}

#[derive(Clone, Debug, Default, derive_more::From)]
pub enum LogDestination {
	#[default]
	Stdout,
	File(PathBuf),
	#[cfg(not(target_arch = "wasm32"))] // no clue why, but `xdg::BaseDirectories` falls apart with it
	Xdg(String),
}
impl LogDestination {
	/// Helper for creating [XdgDataHome](LogDestination::Xdg) variant
	#[cfg(not(target_arch = "wasm32"))]
	pub fn xdg<S: Into<String>>(name: S) -> Self {
		LogDestination::Xdg(name.into())
	}
}
impl From<&str> for LogDestination {
	fn from(s: &str) -> Self {
		if s == "stdout" { LogDestination::Stdout } else { LogDestination::File(s.into()) }
	}
}

fn filter_with_directives(logs_during_init: &mut Vec<Box<dyn FnOnce()>>) -> EnvFilter {
	static DEFAULT_DIRECTIVES: &str = "debug,hyper=info,hyper_util=info";
	static DIRECTIVES_PATH: &str = ".cargo/log_directives";

	let directives = std::fs::read_to_string(DIRECTIVES_PATH).map(Cow::Owned).unwrap_or_else(|_| {
		logs_during_init.push(Box::new(|| warn!("Couldn't read log directives from `{DIRECTIVES_PATH}`, defaulting to default")));
		Cow::Borrowed(DEFAULT_DIRECTIVES)
	});

	let directives_str = directives.clone();
	logs_during_init.push(Box::new(move || info!("Proceeding with following log directives:\n{directives_str}")));

	EnvFilter::builder()
		.parse(&directives)
		.unwrap_or_else(|_| panic!("Error parsing tracing directives:\n```\n{directives}\n```\n"))
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
