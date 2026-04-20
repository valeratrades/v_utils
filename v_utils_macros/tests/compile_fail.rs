fn main() -> ui_test::color_eyre::Result<()> {
	let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
	let proc_macro = find_proc_macro();

	let mut config = ui_test::Config::rustc(root.join("tests/compile_fail"));
	config.program.args.push("--extern".into());
	config.program.args.push(format!("v_utils_macros={}", proc_macro.display()).into());
	config.bless_command = Some("cargo test --test compile_fail -- --bless".to_string());
	config.path_stderr_filter(root, "$DIR");

	ui_test::run_tests(config)
}

fn find_proc_macro() -> std::path::PathBuf {
	let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("CARGO_MANIFEST_DIR has parent");
	let deps_dir = workspace_root.join("target/debug/deps");

	let mut candidates: Vec<_> = std::fs::read_dir(&deps_dir)
		.unwrap_or_else(|_| panic!("target deps dir not found: {}", deps_dir.display()))
		.filter_map(|e| e.ok())
		.map(|e| e.path())
		.filter(|p| {
			let name = p.file_name().unwrap_or_default().to_string_lossy();
			name.starts_with("libv_utils_macros") && (name.ends_with(".so") || name.ends_with(".dylib") || name.ends_with(".dll"))
		})
		.collect();

	candidates.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
	candidates.pop().unwrap_or_else(|| panic!("compiled v_utils_macros not found in {}", deps_dir.display()))
}
