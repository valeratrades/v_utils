/// Returns the home directory path, platform-aware.
/// On Unix: `$HOME`
/// On Windows: `%USERPROFILE%`
#[inline]
pub fn home_dir() -> String {
	#[cfg(windows)]
	{
		std::env::var("USERPROFILE").expect("USERPROFILE environment variable not set")
	}
	#[cfg(not(windows))]
	{
		std::env::var("HOME").expect("HOME environment variable not set")
	}
}

/// Returns the XDG config home fallback path, platform-aware.
/// On Unix: `$XDG_CONFIG_HOME` or `$HOME/.config`
/// On Windows: `%APPDATA%`
#[inline]
pub fn xdg_config_fallback() -> String {
	std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
		#[cfg(windows)]
		{
			std::env::var("APPDATA").expect("APPDATA environment variable not set")
		}
		#[cfg(not(windows))]
		{
			format!("{}/.config", home_dir())
		}
	})
}

/// Returns the XDG data home fallback path, platform-aware.
/// On Unix: `$XDG_DATA_HOME` or `$HOME/.local/share`
/// On Windows: `%LOCALAPPDATA%`
#[inline]
pub fn xdg_data_fallback() -> String {
	std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
		#[cfg(windows)]
		{
			std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA environment variable not set")
		}
		#[cfg(not(windows))]
		{
			format!("{}/.local/share", home_dir())
		}
	})
}

/// Returns the XDG cache home fallback path, platform-aware.
/// On Unix: `$XDG_CACHE_HOME` or `$HOME/.cache`
/// On Windows: `%LOCALAPPDATA%\cache`
#[inline]
pub fn xdg_cache_fallback() -> String {
	std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
		#[cfg(windows)]
		{
			format!("{}/cache", std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA environment variable not set"))
		}
		#[cfg(not(windows))]
		{
			format!("{}/.cache", home_dir())
		}
	})
}

/// Returns the XDG state home fallback path, platform-aware.
/// On Unix: `$XDG_STATE_HOME` or `$HOME/.local/state`
/// On Windows: `%LOCALAPPDATA%\state`
#[inline]
pub fn xdg_state_fallback() -> String {
	std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| {
		#[cfg(windows)]
		{
			format!("{}/state", std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA environment variable not set"))
		}
		#[cfg(not(windows))]
		{
			format!("{}/.local/state", home_dir())
		}
	})
}

/// Returns the XDG runtime dir fallback path, platform-aware.
/// On Unix: `$XDG_RUNTIME_DIR` or `$HOME/.runtime`
/// On Windows: `%TEMP%`
#[inline]
pub fn xdg_runtime_fallback() -> String {
	std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
		#[cfg(windows)]
		{
			std::env::var("TEMP").expect("TEMP environment variable not set")
		}
		#[cfg(not(windows))]
		{
			format!("{}/.runtime", home_dir())
		}
	})
}

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
	/// Will create $XDG_DATA_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	#[macro_export]
	macro_rules! xdg_data_dir {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_data_fallback();
			let mut dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_DATA_HOME/<crate_name>/ and return the path to the file specified in $subpath
	#[macro_export]
	macro_rules! xdg_data_file {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_data_fallback();
			let base_dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			let path = std::path::PathBuf::from($subpath);
			let parent = path.parent().unwrap_or(std::path::Path::new(""));
			let dir = if parent.as_os_str().is_empty() { base_dir.clone() } else { base_dir.join(parent) };
			std::fs::create_dir_all(&dir).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_CONFIG_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	#[macro_export]
	macro_rules! xdg_config_dir {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_config_fallback();
			let mut dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_CONFIG_HOME/<crate_name>/ and return the path to the file specified in $subpath
	#[macro_export]
	macro_rules! xdg_config_file {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_config_fallback();
			let base_dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			let path = std::path::PathBuf::from($subpath);
			let parent = path.parent().unwrap_or(std::path::Path::new(""));
			let dir = if parent.as_os_str().is_empty() { base_dir.clone() } else { base_dir.join(parent) };
			std::fs::create_dir_all(&dir).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_CACHE_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	#[macro_export]
	macro_rules! xdg_cache_dir {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_cache_fallback();
			let mut dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_CACHE_HOME/<crate_name>/ and return the path to the file specified in $subpath
	#[macro_export]
	macro_rules! xdg_cache_file {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_cache_fallback();
			let base_dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			let path = std::path::PathBuf::from($subpath);
			let parent = path.parent().unwrap_or(std::path::Path::new(""));
			let dir = if parent.as_os_str().is_empty() { base_dir.clone() } else { base_dir.join(parent) };
			std::fs::create_dir_all(&dir).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_STATE_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	#[macro_export]
	macro_rules! xdg_state_dir {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_state_fallback();
			let mut dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_STATE_HOME/<crate_name>/ and return the path to the file specified in $subpath
	#[macro_export]
	macro_rules! xdg_state_file {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_state_fallback();
			let base_dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			let path = std::path::PathBuf::from($subpath);
			let parent = path.parent().unwrap_or(std::path::Path::new(""));
			let dir = if parent.as_os_str().is_empty() { base_dir.clone() } else { base_dir.join(parent) };
			std::fs::create_dir_all(&dir).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_RUNTIME_DIR/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	#[macro_export]
	macro_rules! xdg_runtime_dir {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_runtime_fallback();
			let mut dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_RUNTIME_DIR/<crate_name>/ and return the path to the file specified in $subpath
	#[macro_export]
	macro_rules! xdg_runtime_file {
		($subpath:expr) => {{
			let base_path = $crate::io::xdg::xdg_runtime_fallback();
			let base_dir = std::path::PathBuf::from(base_path).join(env!("CARGO_PKG_NAME"));
			let path = std::path::PathBuf::from($subpath);
			let parent = path.parent().unwrap_or(std::path::Path::new(""));
			let dir = if parent.as_os_str().is_empty() { base_dir.clone() } else { base_dir.join(parent) };
			std::fs::create_dir_all(&dir).unwrap();
			base_dir.join(&path)
		}};
	}
}
