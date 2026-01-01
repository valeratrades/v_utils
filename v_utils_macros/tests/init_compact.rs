use std::str::FromStr;

use v_utils_macros::{CompactFormatMap, CompactFormatNamed};

#[derive(CompactFormatNamed, Debug, PartialEq)]
pub struct TrailingStop {
	pub percent: f64,
	pub some_other_field: u32,
}

#[derive(CompactFormatNamed, Debug, PartialEq)]
pub struct Empty {}

#[derive(Clone, CompactFormatMap, Debug, PartialEq)]
pub struct Position {
	pub take_profit: f64,
	pub stop_loss: f64,
}

#[derive(CompactFormatNamed, Debug, PartialEq)]
pub struct Order {
	pub position: Position,
	pub count: u32,
}

// To match your example: '{p=tpsl:t0.4884:s0.5190;c=50%}'
// Outer is CompactFormatMap, inner `p` field is CompactFormatNamed
#[derive(Clone, CompactFormatNamed, Debug, PartialEq)]
pub struct TpSl {
	pub take_profit: f64,
	pub stop_loss: f64,
}

#[derive(CompactFormatMap, Debug, PartialEq)]
pub struct PositionParams {
	pub price: TpSl,
	pub count: u32,
}

fn main() {
	{
		let ts = TrailingStop { percent: 0.5, some_other_field: 42 };
		let ts_write = ts.to_string();
		let ts_read = TrailingStop::from_str(&ts_write).unwrap();
		assert_eq!(ts, ts_read);
	}

	{
		let ts_str = "ts:p-0.5:s42";
		let ts_read = TrailingStop::from_str(ts_str).unwrap();
		assert_eq!(
			ts_read,
			TrailingStop {
				percent: -0.5,
				some_other_field: 42
			}
		);
		let ts_write = ts_read.to_string();
		assert_eq!(ts_str, ts_write);
	}

	{
		let empty_str = "empty";
		let empty_read = Empty::from_str(empty_str).unwrap();
		assert_eq!(empty_read, Empty {});
		let empty_write = empty_read.to_string();
		insta::assert_snapshot!(empty_write, @r###"empty"###);

		let empty_str_colon_nothing = "empty:";
		let empty_str_explicit = "empty:_";
		assert_eq!(Empty::from_str(empty_str_colon_nothing).unwrap(), empty_read);
		assert_eq!(Empty::from_str(empty_str_explicit).unwrap(), empty_read);
	}

	// Test CompactFormatMap
	{
		let pos = Position {
			take_profit: 0.4884,
			stop_loss: 0.519,
		};
		let pos_str = pos.to_string();
		insta::assert_snapshot!(pos_str, @r###"{t=0.4884;s=0.519}"###);

		let pos_read = Position::from_str(&pos_str).unwrap();
		assert_eq!(pos, pos_read);

		// Test parsing with different order of keys
		let pos_reordered = Position::from_str("{s=0.3;t=0.5}").unwrap();
		assert_eq!(pos_reordered, Position { take_profit: 0.5, stop_loss: 0.3 });
	}

	// Test nesting: CompactFormatMap inside CompactFormatNamed
	{
		let order = Order {
			position: Position {
				take_profit: 0.4884,
				stop_loss: 0.519,
			},
			count: 50,
		};
		let order_str = order.to_string();
		insta::assert_snapshot!(order_str, @r###"order:p{t=0.4884;s=0.519}:c50"###);

		let order_read = Order::from_str(&order_str).unwrap();
		assert_eq!(order, order_read);
	}

	// Test nesting: CompactFormatNamed inside CompactFormatMap
	// This matches the example: '{p=tpsl:t0.4884:s0.5190;c=50}'
	{
		let params = PositionParams {
			price: TpSl {
				take_profit: 0.4884,
				stop_loss: 0.519,
			},
			count: 50,
		};
		let params_str = params.to_string();
		insta::assert_snapshot!(params_str, @r###"{p=ts:t0.4884:s0.519;c=50}"###);

		let params_read = PositionParams::from_str(&params_str).unwrap();
		assert_eq!(params, params_read);
	}
}
