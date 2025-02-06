pub use xdg;

#[macro_export]
macro_rules! create_xdg {
	($dir_type:ident $(, $subpath:expr)?) => {{
		let dirs = v_utils::io::xdg::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap(); // CARGO_PKG_NAME will be evaluated to the name of the _crate that calls_ this macro
		let subpath = Option::from($($subpath)?).unwrap_or("");
		match stringify!($dir_type) {
			"state" => dirs.create_state_directory(subpath),
			"data" => dirs.create_data_directory(subpath),
			"cache" => dirs.create_cache_directory(subpath),
			"runtime" => dirs.create_runtime_directory(subpath),
			"config" => dirs.create_config_directory(subpath),
			_ => unimplemented!("Unknown directory type"),
		}
		.unwrap()
	}};
}
