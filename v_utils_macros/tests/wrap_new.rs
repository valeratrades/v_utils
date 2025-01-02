use insta::assert_debug_snapshot;
use v_utils_macros::WrapNew;

#[derive(Debug, Default, PartialEq)]
pub struct MockClient {
	pub initialized: bool,
}

impl MockClient {
	pub fn new() -> Self {
		Self { initialized: true }
	}
}

#[derive(Debug, WrapNew)]
pub struct MockBinance(pub MockClient);

#[test]
fn test_new_wrapper() {
	let binance = MockBinance::new();
	assert_debug_snapshot!(binance, @r#"
 MockBinance(
     MockClient {
         initialized: true,
     },
 )
 "#);
}
