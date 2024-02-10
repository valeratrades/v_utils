#[test]
fn tests() {
	let t = trybuild::TestCases::new();
	t.pass("tests/01-graphemics.rs");
	t.pass("tests/02-derive.rs");
}
