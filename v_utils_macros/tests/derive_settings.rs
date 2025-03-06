use serde::{Deserialize, Serialize};
//use v_utils::io::ExpandedPath;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, v_utils_macros::Settings)]
struct Settings {
	pub mock: bool,
	pub positions_dir: std::path::PathBuf,
	pub binance: Binance,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, v_utils_macros::MyConfigPrimitives, v_utils_macros::Settings)]
struct Binance {
	pub read_key: String,
	pub read_secret: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Cli {
	config: Option<std::path::PathBuf>, //TODO: switch to ExpandedPath
	//#[flatten]
	//settings: SettingsArgs,
	mock: bool,
	positions_dir: Option<std::path::PathBuf>,
	binance_read_key: Option<String>,
	binance_read_secret: Option<String>,
}

// needs to gen:
// 4 sources
// 1. config file (using `config` crate)
// 2. env vars (like APPNAME_MOCK, APPNAME_BINANCE_FULL_KEY, etc.)
// 3. clap flags (like --config, --mock, --positions-dir, --binance-full-key, etc.)
// 4. cached setting values (stored in XDG_STATE_HOME/appname/settings.json). We cache only fields with `#[default]` on them (otherwise will be inconistent first-start behavior)
// Hierarchy (descending importance): flags -> config -> env -> cached

//NB: each component that does not explicitly specify `#[default]` is required, and not being able to derive it from at least one of the sorces leads to

//NB: clap part needs to come with `flatten` (otherwise can't ensure correct position of --config flag for providing its source path)
// so then it also naturally becomes a macro

//impl plan:
//+ build fn (start with just conf files)
//+ integration test
//+ flags
//+ #[default]
//* cached
//+Q: integrate with MyConfigPrimitives?

//Q: it is possible that my current MyConfigPrimitives derive is messing with aggregated construction, as it derives `deserialize` instead of `try_deserializes`

//Q: n
fn main() {
	// should be full integration: use clap, create an actual partial conf file in `/tmp`, add env flags, create a clap string, then aggregate and attemtp creating.

	std::fs::write(
		"/tmp/test.toml",
		r#"
		positions_dir = "/tmp"
		[binance]
		read_secret = { env = "BINANCE_READ_SECRET" }
		#read_secret = "written out read_secret"
		"#,
	)
	.unwrap();
	std::env::set_var("BINANCE_READ_SECRET", "isarendtiaeahoulegf");

	std::env::set_var("V_UTILS_MACROS:MOCK", "false");
	std::env::set_var("V_UTILS_MACROS:BINANCE.READ_KEY", "env_read_key"); //NB: notice that nesting is provided by `.`, and prefix_separator is `:`

	let settings = Settings::try_build(Some("/tmp/test.toml".into())).unwrap();
	insta::assert_debug_snapshot!(settings, @r#"
 Settings {
     mock: false,
     positions_dir: "/tmp",
     binance: Binance {
         read_key: "env_read_key",
         read_secret: "written out read_secret",
     },
 }
 "#);
}
