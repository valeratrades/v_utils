#[cfg(feature = "xdg")]
pub use xdg_with_lib::*;
#[cfg(feature = "xdg")]
mod xdg_with_lib {
	macro_rules! impl_xdg_fn {
		($fn_name:ident, $dir_type:ident) => {
			#[doc = concat!("Will create ", stringify!($fn_name), "_home/<crate_name>/$subpath/ (\"\" for no subpath; subpath is a **DIR**)")]
			#[macro_export]
			macro_rules! $fn_name {
				($subpath: expr) => {{
					let dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap();
					dirs.$dir_type($subpath).unwrap()
				}};
				() => {
					$fn_name!("")
				};
			}
		};
	}

	impl_xdg_fn!(xdg_data, create_data_directory);
	impl_xdg_fn!(xdg_config, create_config_directory);
	impl_xdg_fn!(xdg_cache, create_cache_directory);
	impl_xdg_fn!(xdg_state, create_state_directory);
	impl_xdg_fn!(xdg_runtime, create_runtime_directory);
}

#[cfg(not(feature = "xdg"))]
pub use xdg_no_deps::*;
#[cfg(not(feature = "xdg"))]
mod xdg_no_deps {
	macro_rules! impl_backup_xdg_fn {
		($method_name:ident, $env_var:expr, $fallback_dir:expr) => {
			#[doc = concat!("Will create $", stringify!($env_var), "/<crate_name>/$subpath/ (\"\" for no subpath; subpath is a **DIR**)")]
			#[macro_export]
			macro_rules! $method_name {
				($subpath: expr) => {{
					let base_path = std::env::var($env_var).unwrap_or_else(|_| format!("{}/{}", std::env::var("HOME").unwrap(), $fallback_dir));
					let mut dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
					if !$subpath.is_empty() {
						dir = dir.join($subpath);
					}
					std::fs::create_dir_all(&dir).unwrap();
					dir
				}};
			}
		};
	}

	impl_backup_xdg_fn!(xdg_data, "XDG_DATA_HOME", ".local/share");
	impl_backup_xdg_fn!(xdg_config, "XDG_CONFIG_HOME", ".config");
	impl_backup_xdg_fn!(xdg_cache, "XDG_CACHE_HOME", ".cache");
	impl_backup_xdg_fn!(xdg_state, "XDG_STATE_HOME", ".local/state");
	impl_backup_xdg_fn!(xdg_runtime, "XDG_RUNTIME_DIR", ".runtime");
}
