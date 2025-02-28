pub extern crate xdg;

macro_rules! impl_xdg_fn {
	($fn_name:ident, $dir_type:ident) => {
		#[doc = concat!("Will create ", stringify!($fn_name), "_home/<crate_name>/$subpath (\"\" for no subpath)")]
		pub fn $fn_name(subpath: &str) -> std::path::PathBuf {
			let dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap();
			dirs.$dir_type(subpath).unwrap()
		}
	};
}

impl_xdg_fn!(xdg_data, create_data_directory);
impl_xdg_fn!(xdg_config, create_config_directory);
impl_xdg_fn!(xdg_cache, create_cache_directory);
impl_xdg_fn!(xdg_state, create_state_directory);
impl_xdg_fn!(xdg_runtime, create_runtime_directory);
