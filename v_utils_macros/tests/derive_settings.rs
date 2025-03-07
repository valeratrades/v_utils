use serde::{Deserialize, Serialize};
//use v_utils::io::ExpandedPath;
use v_utils::__internal::config;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize, v_utils_macros::Settings)]
struct Settings {
	pub mock: bool,
	pub positions_dir: std::path::PathBuf,
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
	pub read_key: String,
	pub read_secret: String,
}

#[derive(Debug, Default, PartialEq, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	//config: Option<std::path::PathBuf>, //TODO: switch to ExpandedPath
	#[clap(flatten)]
	settings: SettingsFlags,
}

impl config::Source for SettingsFlags {
	fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
		Box::new((*self).clone())
	}

	/// Collect all configuration properties available from this source into
	/// a [`Map`].
	fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
		let mut map = config::Map::new();
		if let Some(bybit_read_secret) = &self.bybit.bybit_read_secret {
			map.insert(
				"bybit.read_secret".to_owned(),
				config::Value::new(Some(&"flags:bybit".to_owned()), config::ValueKind::String(bybit_read_secret.to_owned())),
			);
		}
		Ok(map)
	}
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
//- [ ] #[default]
//- [ ] cached

//Q: it is possible that my current MyConfigPrimitives derive is messing with aggregated construction, as it derives `deserialize` instead of `try_deserializes`

//Q: n
fn main() {
	// should be full integration: use clap, create an actual partial conf file in `/tmp`, add env flags, create a clap string, then aggregate and attemtp creating.

	std::fs::write(
		"/tmp/test.toml",
		r#"
		positions_dir = "/tmp/"
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

	let cli_input = vec!["", "--config", "/tmp/test.toml", "--bybit-read-secret", "passed as a flag"]; // should follow std::env::os_args()
	use clap::Parser as _;
	let cli = Cli::parse_from(cli_input);
	dbg!(&cli);

	let out_settings = Settings::try_build(cli.settings).unwrap();
	insta::assert_debug_snapshot!(out_settings, @r#"
 Settings {
     mock: false,
     positions_dir: "/tmp/",
     binance: Binance {
         read_key: "env_read_key",
         read_secret: "isarendtiaeahoulegf",
     },
     bybit: Bybit {
         read_key: "placeholder",
         read_secret: "passed as a flag",
     },
 }
 "#);
}
