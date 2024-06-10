#![allow(dead_code, unused_imports)]
use tracing::info;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer, Type};
use tracing_subscriber::{
	fmt::{self, MakeWriter},
	layer::SubscriberExt,
	EnvFilter, Registry,
};

///# Panics
pub fn init_subscriber() {
	{
		let mut rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "".to_owned());
		if !rust_log.is_empty() {
			rust_log.push(',');
		}
		rust_log.push_str("tokio=trace,runtime=trace");
		unsafe { std::env::set_var("RUST_LOG", rust_log) };
	}

	let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));

	// Fucking rust. And No, you can't make this shit with any less duplication, without sacrificing your soul.
	// Only difference is `std::io::{stdout,sink}`
	if std::env::var("TEST_LOG").is_ok() {
		let formatting_layer = BunyanFormattingLayer::new("discretionary_engine".into(), std::io::stdout);
		let subscriber = Registry::default().with(env_filter).with(JsonStorageLayer).with(formatting_layer);
		set_global_default(subscriber).expect("Failed to set subscriber");
	} else {
		let formatting_layer = BunyanFormattingLayer::new("discretionary_engine".into(), std::io::sink);
		let subscriber = Registry::default().with(env_filter).with(JsonStorageLayer).with(formatting_layer);
		set_global_default(subscriber).expect("Failed to set subscriber");
	}

	//let formatting_layer = fmt::layer().json().pretty().with_writer(std::io::stdout);
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
