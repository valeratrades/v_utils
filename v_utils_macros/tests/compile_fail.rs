use std::path::{Path, PathBuf};

use ui_test::{dependencies::DependencyBuilder, spanned::Spanned};

fn main() -> ui_test::color_eyre::Result<()> {
	let root = Path::new(env!("CARGO_MANIFEST_DIR"));

	// Strict suite: every diagnostic must be annotated (these pin exact macro-expansion errors).
	let strict = base_config(root.join("tests/compile_fail"), root);
	// Cascade suite: the `SettingsNested` trait-bound failure necessarily repeats across every
	// site that names `<T as SettingsNested>::Flags`. Annotating each is brittle, so this dir
	// runs with annotations off and pins the full `.stderr` snapshot instead.
	let mut cascade = base_config(root.join("tests/compile_fail_cascade"), root);
	cascade.comment_defaults.base().require_annotations = Spanned::dummy(false).into();

	ui_test::run_tests(strict)?;
	ui_test::run_tests(cascade)
}

fn base_config(dir: PathBuf, root: &Path) -> ui_test::Config {
	let mut config = ui_test::Config::rustc(dir);
	// Resolve extern crates through cargo from a side manifest, so a case can reference both
	// `v_utils_macros` and the `cli`-featured `v_utils` facade (the latter is needed for the
	// `SettingsNested` trait-bound diagnostic, which only surfaces at type-check time in code
	// the macro emits against `v_utils::...` paths). Hand-picking rlibs cannot pin features.
	config.comment_defaults.base().set_custom(
		"dependencies",
		DependencyBuilder {
			crate_manifest_path: root.join("tests/compile_fail_deps/Cargo.toml"),
			..DependencyBuilder::default()
		},
	);
	config.bless_command = Some("cargo test --test compile_fail -- --bless".to_string());
	config.path_stderr_filter(root, "$DIR");
	config
}
