use insta::assert_debug_snapshot;

#[derive(Debug, Default, PartialEq, v_utils_macros::Settings)]
pub struct Settings {
	pub mock: bool,
	pub positions_dir: PathBuf,
	pub binance: Binance,
}
#[derive(Clone, Debug, v_util_macros::Settings)]
pub struct Binance {
	pub full_key: String,
	pub full_secret: String,
	pub read_key: String,
	pub read_secret: String,
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
//+ conf
//+ env
//+ flags
//+ build fn
//+ #[default]
//* cached

fn main() {
	// should be full integration: use clap, create an actual partial conf file in `/tmp`, add env flags, create a clap string, then aggregate and attemtp creating.
	assert!(true); //TODO .
}
