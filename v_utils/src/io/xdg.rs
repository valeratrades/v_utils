pub use xdg;

#[macro_export]
macro_rules! state_dir {
	() => {
		v_utils::io::xdg::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))
			.unwrap()
			.create_state_directory("")
			.unwrap();
	};
}

#[macro_export]
macro_rules! share_dir {
	() => {
		v_utils::io::xdg::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))
			.unwrap()
			.create_data_directory("")
			.unwrap();
	};
}
