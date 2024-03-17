#[test]
fn tests() {
	let t = trybuild::TestCases::new();
	//t.pass("tests/protocols_in_discetionary_engine.rs");
	t.pass("tests/ask_claude.rs");
}
