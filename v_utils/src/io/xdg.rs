#[cfg(feature = "xdg")]
mod xdg_with_lib {
	macro_rules! impl_xdg_dir_fn {
		($fn_name:ident, $dir_type:ident) => {
			#[doc = concat!("Will create ", stringify!($fn_name), "_home/<crate_name>/$subpath/ (\"\" for no subpath; subpath is a **DIR**)")]
			#[macro_export]
			macro_rules! $fn_name {
				($subpath: expr) => {{
					let dirs = $crate::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
					dirs.$dir_type($subpath).unwrap()
				}};
				() => {
					$fn_name!("")
				};
			}
		};
	}

	macro_rules! impl_xdg_file_fn {
		($fn_name:ident, $dir_type:ident) => {
			#[doc = concat!("Will create ", stringify!($fn_name), "_home/<crate_name>/ and return the path to the file specified in $subpath")]
			#[macro_export]
			macro_rules! $fn_name {
				($subpath: expr) => {{
					let dirs = $crate::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
					let path = std::path::PathBuf::from($subpath);
					let parent = path.parent().unwrap_or(std::path::Path::new(""));
					let base_dir = dirs.$dir_type(parent).unwrap();
					base_dir.join(path.file_name().unwrap())
				}};
			}
		};
	}

	impl_xdg_dir_fn!(xdg_data_dir, create_data_directory);
	impl_xdg_file_fn!(xdg_data_file, create_data_directory);
	impl_xdg_dir_fn!(xdg_config_dir, create_config_directory);
	impl_xdg_file_fn!(xdg_config_file, create_config_directory);
	impl_xdg_dir_fn!(xdg_cache_dir, create_cache_directory);
	impl_xdg_file_fn!(xdg_cache_file, create_cache_directory);
	impl_xdg_dir_fn!(xdg_state_dir, create_state_directory);
	impl_xdg_file_fn!(xdg_state_file, create_state_directory);
	impl_xdg_dir_fn!(xdg_runtime_dir, create_runtime_directory);
	impl_xdg_file_fn!(xdg_runtime_file, create_runtime_directory);
}

#[cfg(not(feature = "xdg"))]
mod xdg_no_deps {
	macro_rules! impl_backup_xdg_dir_fn {
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

	macro_rules! impl_backup_xdg_file_fn {
		($method_name:ident, $env_var:expr, $fallback_dir:expr) => {
			#[doc = concat!("Will create $", stringify!($env_var), "/<crate_name>/ and return the path to the file specified in $subpath")]
			#[macro_export]
			macro_rules! $method_name {
				($subpath: expr) => {{
					let base_path = std::env::var($env_var).unwrap_or_else(|_| format!("{}/{}", std::env::var("HOME").unwrap(), $fallback_dir));
					let base_dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
					let path = std::path::PathBuf::from($subpath);
					let parent = path.parent().unwrap_or(std::path::Path::new(""));
					let dir = if parent.as_os_str().is_empty() { base_dir.clone() } else { base_dir.join(parent) };
					std::fs::create_dir_all(&dir).unwrap();
					base_dir.join(&path)
				}};
			}
		};
	}

	impl_backup_xdg_dir_fn!(xdg_data_dir, "XDG_DATA_HOME", ".local/share");
	impl_backup_xdg_file_fn!(xdg_data_file, "XDG_DATA_HOME", ".local/share");
	impl_backup_xdg_dir_fn!(xdg_config_dir, "XDG_CONFIG_HOME", ".config");
	impl_backup_xdg_file_fn!(xdg_config_file, "XDG_CONFIG_HOME", ".config");
	impl_backup_xdg_dir_fn!(xdg_cache_dir, "XDG_CACHE_HOME", ".cache");
	impl_backup_xdg_file_fn!(xdg_cache_file, "XDG_CACHE_HOME", ".cache");
	impl_backup_xdg_dir_fn!(xdg_state_dir, "XDG_STATE_HOME", ".local/state");
	impl_backup_xdg_file_fn!(xdg_state_file, "XDG_STATE_HOME", ".local/state");
	impl_backup_xdg_dir_fn!(xdg_runtime_dir, "XDG_RUNTIME_DIR", ".runtime");
	impl_backup_xdg_file_fn!(xdg_runtime_file, "XDG_RUNTIME_DIR", ".runtime");
}
