#[test]
fn tests() {
	let t = trybuild::TestCases::new();
	t.pass("tests/graphemics.rs");
	t.pass("tests/from_vec_str.rs");
	t.pass("tests/init_compact.rs");
	t.pass("tests/my_config_primitives.rs");
	t.pass("tests/make_df.rs");
}
