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

#[cfg(unix)]
pub(crate) mod app_index {
	use std::{fs, io::ErrorKind, path::Path};

	/// Links `~/.<app>/<entry>` to `target`, so every path created for an app is reachable from one place.
	pub fn index(app: &str, entry: &str, target: &Path) {
		let index_dir = Path::new(&super::home_dir()).join(format!(".{app}"));
		fs::create_dir_all(&index_dir).unwrap_or_else(|e| panic!("creating {}: {e}", index_dir.display()));
		let link = index_dir.join(entry);

		match fs::symlink_metadata(&link) {
			Err(e) if e.kind() == ErrorKind::NotFound => {}
			Err(e) => panic!("stat {}: {e}", link.display()),
			Ok(meta) if meta.is_symlink() => {
				let old = fs::read_link(&link).unwrap_or_else(|e| panic!("read_link {}: {e}", link.display()));
				if old == target {
					return;
				}
				assert!(
					!old.exists(),
					"{} points at {}, which still holds data, but {} is now expected there; move it by hand",
					link.display(),
					old.display(),
					target.display()
				);
				fs::remove_file(&link).unwrap_or_else(|e| panic!("removing stale {}: {e}", link.display()));
			}
			Ok(_) => migrate_legacy(&link, target),
		}

		if let Err(e) = std::os::unix::fs::symlink(target, &link) {
			let raced_to_same = e.kind() == ErrorKind::AlreadyExists && fs::read_link(&link).is_ok_and(|p| p == target); // another process of the same app got here first
			assert!(raced_to_same, "symlink {} -> {}: {e}", link.display(), target.display());
		}
	}

	fn migrate_legacy(link: &Path, target: &Path) {
		let target_empty = match fs::symlink_metadata(target) {
			Err(e) if e.kind() == ErrorKind::NotFound => true,
			Err(e) => panic!("stat {}: {e}", target.display()),
			Ok(m) if m.is_dir() => fs::read_dir(target).unwrap_or_else(|e| panic!("read_dir {}: {e}", target.display())).next().is_none(),
			Ok(m) => m.len() == 0,
		};
		assert!(
			target_empty,
			"legacy {} and {} both hold data; merge them by hand, then remove {}",
			link.display(),
			target.display(),
			link.display()
		);
		if target.is_dir() {
			fs::remove_dir(target).unwrap_or_else(|e| panic!("removing empty {}: {e}", target.display()));
		} else if target.exists() {
			fs::remove_file(target).unwrap_or_else(|e| panic!("removing empty {}: {e}", target.display()));
		}
		// ponytail: no cross-fs copy, add if a real migration hits it
		if let Err(e) = fs::rename(link, target) {
			match e.kind() {
				ErrorKind::CrossesDevices => panic!("{} and {} are on different filesystems; move manually", link.display(), target.display()),
				_ => panic!("moving {} -> {}: {e}", link.display(), target.display()),
			}
		}
	}
}

#[cfg(feature = "xdg")]
mod xdg_with_lib {
	macro_rules! impl_xdg_dir_fn {
		($fn_name:ident, $dir_type:ident, $kind:literal) => {
			#[doc = concat!("Will create ", stringify!($fn_name), "_home/<crate_name>/$subpath/ (\"\" for no subpath; subpath is a **DIR**)")]
			#[doc = concat!("On unix, also links `~/.<crate_name>/", $kind, "` to the base dir.")]
			#[macro_export]
			macro_rules! $fn_name {
				($subpath: expr) => {{
					let dirs = $crate::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
					#[cfg(unix)]
					$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), $kind, &dirs.$dir_type("").unwrap());
					dirs.$dir_type($subpath).unwrap()
				}};
				() => {
					$fn_name!("")
				};
			}
		};
	}

	macro_rules! impl_xdg_file_fn {
		($fn_name:ident, $dir_type:ident, $kind:literal) => {
			#[doc = concat!("Will create ", stringify!($fn_name), "_home/<crate_name>/ and return the path to the file specified in $subpath")]
			#[doc = concat!("On unix, also links `~/.<crate_name>/", $kind, "` to the base dir.")]
			#[macro_export]
			macro_rules! $fn_name {
				($subpath: expr) => {{
					let dirs = $crate::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
					#[cfg(unix)]
					$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), $kind, &dirs.$dir_type("").unwrap());
					let path = std::path::PathBuf::from($subpath);
					let parent = path.parent().unwrap_or(std::path::Path::new(""));
					let base_dir = dirs.$dir_type(parent).unwrap();
					base_dir.join(path.file_name().unwrap())
				}};
			}
		};
	}

	impl_xdg_dir_fn!(xdg_data_dir, create_data_directory, "data");
	impl_xdg_file_fn!(xdg_data_file, create_data_directory, "data");
	impl_xdg_dir_fn!(xdg_config_dir, create_config_directory, "config");
	impl_xdg_file_fn!(xdg_config_file, create_config_directory, "config");
	impl_xdg_dir_fn!(xdg_cache_dir, create_cache_directory, "cache");
	impl_xdg_file_fn!(xdg_cache_file, create_cache_directory, "cache");
	impl_xdg_dir_fn!(xdg_state_dir, create_state_directory, "state");
	impl_xdg_file_fn!(xdg_state_file, create_state_directory, "state");
	impl_xdg_dir_fn!(xdg_runtime_dir, create_runtime_directory, "runtime");
	impl_xdg_file_fn!(xdg_runtime_file, create_runtime_directory, "runtime");
}

#[cfg(not(feature = "xdg"))]
mod xdg_no_deps {
	/// Will create $XDG_DATA_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	/// On unix, also links `~/.<crate_name>/data` to the base dir.
	#[macro_export]
	macro_rules! xdg_data_dir {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_data_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "data", &base_dir);
			let mut dir = base_dir;
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_DATA_HOME/<crate_name>/ and return the path to the file specified in $subpath
	/// On unix, also links `~/.<crate_name>/data` to the base dir.
	#[macro_export]
	macro_rules! xdg_data_file {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_data_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "data", &base_dir);
			let path = std::path::PathBuf::from($subpath);
			std::fs::create_dir_all(base_dir.join(path.parent().unwrap_or(std::path::Path::new("")))).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_CONFIG_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	/// On unix, also links `~/.<crate_name>/config` to the base dir.
	#[macro_export]
	macro_rules! xdg_config_dir {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_config_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "config", &base_dir);
			let mut dir = base_dir;
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_CONFIG_HOME/<crate_name>/ and return the path to the file specified in $subpath
	/// On unix, also links `~/.<crate_name>/config` to the base dir.
	#[macro_export]
	macro_rules! xdg_config_file {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_config_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "config", &base_dir);
			let path = std::path::PathBuf::from($subpath);
			std::fs::create_dir_all(base_dir.join(path.parent().unwrap_or(std::path::Path::new("")))).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_CACHE_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	/// On unix, also links `~/.<crate_name>/cache` to the base dir.
	#[macro_export]
	macro_rules! xdg_cache_dir {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_cache_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "cache", &base_dir);
			let mut dir = base_dir;
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_CACHE_HOME/<crate_name>/ and return the path to the file specified in $subpath
	/// On unix, also links `~/.<crate_name>/cache` to the base dir.
	#[macro_export]
	macro_rules! xdg_cache_file {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_cache_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "cache", &base_dir);
			let path = std::path::PathBuf::from($subpath);
			std::fs::create_dir_all(base_dir.join(path.parent().unwrap_or(std::path::Path::new("")))).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_STATE_HOME/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	/// On unix, also links `~/.<crate_name>/state` to the base dir.
	#[macro_export]
	macro_rules! xdg_state_dir {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_state_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "state", &base_dir);
			let mut dir = base_dir;
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_STATE_HOME/<crate_name>/ and return the path to the file specified in $subpath
	/// On unix, also links `~/.<crate_name>/state` to the base dir.
	#[macro_export]
	macro_rules! xdg_state_file {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_state_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "state", &base_dir);
			let path = std::path::PathBuf::from($subpath);
			std::fs::create_dir_all(base_dir.join(path.parent().unwrap_or(std::path::Path::new("")))).unwrap();
			base_dir.join(&path)
		}};
	}

	/// Will create $XDG_RUNTIME_DIR/<crate_name>/$subpath/ ("" for no subpath; subpath is a **DIR**)
	/// On unix, also links `~/.<crate_name>/runtime` to the base dir.
	#[macro_export]
	macro_rules! xdg_runtime_dir {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_runtime_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "runtime", &base_dir);
			let mut dir = base_dir;
			if !$subpath.is_empty() {
				dir = dir.join($subpath);
			}
			std::fs::create_dir_all(&dir).unwrap();
			dir
		}};
	}

	/// Will create $XDG_RUNTIME_DIR/<crate_name>/ and return the path to the file specified in $subpath
	/// On unix, also links `~/.<crate_name>/runtime` to the base dir.
	#[macro_export]
	macro_rules! xdg_runtime_file {
		($subpath:expr) => {{
			let base_dir = std::path::PathBuf::from($crate::io::xdg::xdg_runtime_fallback()).join(env!("CARGO_PKG_NAME"));
			std::fs::create_dir_all(&base_dir).unwrap();
			#[cfg(unix)]
			$crate::__internal::xdg_index(env!("CARGO_PKG_NAME"), "runtime", &base_dir);
			let path = std::path::PathBuf::from($subpath);
			std::fs::create_dir_all(base_dir.join(path.parent().unwrap_or(std::path::Path::new("")))).unwrap();
			base_dir.join(&path)
		}};
	}
}
