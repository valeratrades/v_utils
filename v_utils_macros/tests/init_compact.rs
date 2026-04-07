#![feature(default_field_values)]

use std::str::FromStr;

use v_utils_macros::{CompactFormatMap, CompactFormatNamed, TryParseVariants};

#[derive(CompactFormatNamed, Debug, PartialEq)]
#[compact(default)]
pub struct StructDefault {
	pub alpha: u32,
	pub beta: u32,
}

impl Default for StructDefault {
	fn default() -> Self {
		Self { alpha: 10, beta: 20 }
	}
}

#[derive(CompactFormatNamed, Debug, PartialEq)]
pub struct FieldDefault {
	pub required: f64,
	#[compact(default)]
	pub optional: u32,
}

#[derive(CompactFormatNamed, Debug, PartialEq)]
pub struct FieldDefaultExpr {
	pub required: f64,
	#[compact(default = 100)]
	pub optional: u32,
}

#[derive(CompactFormatNamed, Debug, Default, PartialEq)]
pub struct InlineDefault {
	pub required: f64,
	pub count: u32 = 42,
}

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

	// Test #[compact(default)] on struct — all fields optional via Self::default()
	{
		let bare = StructDefault::from_str("sd").unwrap();
		assert_eq!(bare, StructDefault { alpha: 10, beta: 20 });

		let partial = StructDefault::from_str("sd:a99").unwrap();
		assert_eq!(partial, StructDefault { alpha: 99, beta: 20 });

		let full = StructDefault::from_str("sd:a5:b6").unwrap();
		assert_eq!(full, StructDefault { alpha: 5, beta: 6 });
	}

	// Test #[compact(default)] on individual fields
	{
		let bare = FieldDefault::from_str("fd:r1.0").unwrap();
		assert_eq!(bare, FieldDefault { required: 1.0, optional: 0 });

		let full = FieldDefault::from_str("fd:r1.0:o42").unwrap();
		assert_eq!(full, FieldDefault { required: 1.0, optional: 42 });

		// Missing required field should fail
		assert!(FieldDefault::from_str("fd").is_err());
	}

	// Test #[compact(default = expr)] on field
	{
		let bare = FieldDefaultExpr::from_str("fde:r1.23").unwrap();
		assert_eq!(bare, FieldDefaultExpr { required: 1.23, optional: 100 });
	}

	// Test inline default field values (`field: Type = expr`)
	{
		let bare = InlineDefault::from_str("id:r2.5").unwrap();
		assert_eq!(bare, InlineDefault { required: 2.5, count: 42 });

		let full = InlineDefault::from_str("id:r2.5:c99").unwrap();
		assert_eq!(full, InlineDefault { required: 2.5, count: 99 });

		// Missing required field should fail
		assert!(InlineDefault::from_str("id").is_err());

		// Round-trip
		let val = InlineDefault { required: 1.0, count: 7 };
		let s = val.to_string();
		let parsed = InlineDefault::from_str(&s).unwrap();
		assert_eq!(val, parsed);
	}

	// Test TryParseVariants — derives FromStr on an enum by trying each variant's inner FromStr
	{
		#[derive(Debug, PartialEq, TryParseVariants)]
		enum Strategy {
			Trailing(TrailingStop),
			Empty(Empty),
		}

		let parsed: Strategy = "ts:p0.5:s42".parse().unwrap();
		assert_eq!(parsed, Strategy::Trailing(TrailingStop { percent: 0.5, some_other_field: 42 }));

		let parsed: Strategy = "empty".parse().unwrap();
		assert_eq!(parsed, Strategy::Empty(Empty {}));

		assert!("unknown:x1".parse::<Strategy>().is_err());
	}
}
