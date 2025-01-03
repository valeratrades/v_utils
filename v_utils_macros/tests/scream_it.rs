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

fn main() {
	let order = OrderType::from_str("STOP_MARKET").unwrap();
	assert_debug_snapshot!(order, @"StopMarket");

	let order_str = OrderType::TakeProfit.to_string();
	assert_debug_snapshot!(order_str, @r#""TAKE_PROFIT""#);

	let invalid_order = OrderType::from_str("INVALID");
	assert_debug_snapshot!(invalid_order, @r#"
 Err(
     (),
 )
 "#);

	let deserialize_order = serde_json::from_str::<OrderType>(r#""TAKE_PROFIT""#).unwrap();
	assert_debug_snapshot!(deserialize_order, @"TakeProfit");

	let serialized_str = serde_json::to_string(&OrderType::TrailingStopMarket).unwrap();
	assert_debug_snapshot!(serialized_str, @r#""\"TRAILING_STOP_MARKET\"""#);
}
