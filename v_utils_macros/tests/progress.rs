#[test]
fn tests() {
	let t = trybuild::TestCases::new();
	t.pass("tests/graphemics.rs");
	t.pass("tests/from-vec-str.rs");
	t.pass("tests/init-compact.rs");
	t.pass("tests/private-values.rs");
}
