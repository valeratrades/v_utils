pub use xdg;

#[macro_export]
macro_rules! create_xdg {
	($dir_type:ident) => {{
		let dirs = v_utils::io::xdg::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap();
		match stringify!($dir_type) {
			"state" => dirs.create_state_directory(""),
			"data" => dirs.create_data_directory(""),
			"cache" => dirs.create_cache_directory(""),
			"runtime" => dirs.create_runtime_directory(""),
			"config" => dirs.create_config_directory(""),
			_ => unimplemented!("Unknown directory type"),
		}
		.unwrap()
	}};
}
