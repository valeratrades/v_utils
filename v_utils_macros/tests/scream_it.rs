use std::str::FromStr;

use insta::assert_debug_snapshot;
use v_utils_macros::ScreamIt;

#[derive(ScreamIt, Debug, PartialEq)]
pub enum OrderType {
	Limit,
	Market,
	Stop,
	StopMarket,
	TakeProfit,
	TakeProfitMarket,
	TrailingStopMarket,
}

#[test]
fn test_scream_it() {
	let order = OrderType::from_str("LIMIT").unwrap();
	assert_debug_snapshot!(order, @"Limit");

	let order_str = OrderType::Market.to_string();
	assert_debug_snapshot!(order_str, @r#""MARKET""#);

	let invalid_order = OrderType::from_str("INVALID");
	assert_debug_snapshot!(invalid_order, @r#"
 Err(
     (),
 )
 "#);
}
