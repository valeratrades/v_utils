pub use xdg;

macro_rules! impl_xdg_fn {
	($fn_name:ident, $dir_type:ident) => {
		#[doc = concat!("Will create ", stringify!($fn_name), "_HOME/<crate_name>/$subpath (\"\" for no subpath)")]
		pub fn $fn_name(subpath: &str) -> std::path::PathBuf {
			let crate_name = env!("CARGO_PKG_NAME");
			let dirs = xdg::BaseDirectories::with_prefix(crate_name).unwrap();
			dirs.$dir_type(subpath).unwrap()
		}
	};
}

impl_xdg_fn!(xdg_data, create_data_directory);
impl_xdg_fn!(xdg_config, create_config_directory);
impl_xdg_fn!(xdg_cache, create_cache_directory);
impl_xdg_fn!(xdg_state, create_state_directory);
impl_xdg_fn!(xdg_runtime, create_runtime_directory);
