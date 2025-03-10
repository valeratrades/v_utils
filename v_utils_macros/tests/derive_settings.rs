use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, v_utils_macros::Settings)]
struct Settings {
	pub mock: bool,
	pub positions_dir: std::path::PathBuf,
	pub sth_else: String,
	pub numeric: f64,
	#[settings(flatten)]
	pub binance: Binance,
	#[settings(flatten)]
	pub bybit: Bybit,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, v_utils_macros::MyConfigPrimitives, v_utils_macros::SettingsBadlyNested)]
struct Binance {
	pub read_key: String,
	pub read_secret: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, v_utils_macros::MyConfigPrimitives, v_utils_macros::SettingsBadlyNested)]
struct Bybit {
	pub secret_path: std::path::PathBuf,
}

#[derive(Debug, Default, PartialEq, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	//config: Option<std::path::PathBuf>, //TODO: switch to ExpandedPath
	#[clap(flatten)]
	settings: SettingsFlags,
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
//- [x] build fn (start with just conf files)
//- [x] integration test
//- [.](left: config::Source impl) flags
//		- [ ] derive while hardcoding ValueKind::String
//		- [ ] proper matching for ValueKind types
//- [ ] #[default] (simply mirror them as clap defaults)
//- [ ] cached

//Q: it is possible that my current MyConfigPrimitives derive is messing with aggregated construction, as it derives `deserialize` instead of `try_deserializes`

//Q: n
fn main() {
	// should be full integration: use clap, create an actual partial conf file in `/tmp`, add env flags, create a clap string, then aggregate and attemtp creating.

	std::fs::write(
		"/tmp/test.toml",
		r#"
		sth_else = "define it here as-is"
		[binance]
		read_secret = { env = "BINANCE_READ_SECRET" }
		[bybit]
		read_key = "placeholder"
		"#,
	)
	.unwrap();
	std::env::set_var("BINANCE_READ_SECRET", "isarendtiaeahoulegf");

	//NB: to represent nesting we use `__` separator
	std::env::set_var("V_UTILS_MACROS__MOCK", "false");
	std::env::set_var("V_UTILS_MACROS__BINANCE__READ_KEY", "env_read_key");

	let cli_input = vec![
		"",
		"--config",
		"/tmp/test.toml",
		"--bybit-secret-path",
		"passed as a flag",
		"--positions-dir",
		"/tmp/please_work/",
		"--numeric",
		"0.682",
	]; // should follow std::env::os_args()
	use clap::Parser as _;
	let cli = Cli::parse_from(cli_input);
	dbg!(&cli);

	let out_settings = Settings::try_build(cli.settings).unwrap();
	insta::assert_debug_snapshot!(out_settings, @r#"
 Settings {
     mock: false,
     positions_dir: "/tmp/please_work/",
     sth_else: "define it here as-is",
     numeric: 0.682,
     binance: Binance {
         read_key: "env_read_key",
         read_secret: "isarendtiaeahoulegf",
     },
     bybit: Bybit {
         secret_path: "passed as a flag",
     },
 }
 "#);
}
