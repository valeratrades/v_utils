use std::str::FromStr;
use v_utils_macros::CompactFormat;

#[derive(CompactFormat, Debug, PartialEq)]
pub struct TrailingStop {
	pub percent: f64,
	pub some_other_field: u32,
}

fn main() {
	{
		let ts = TrailingStop {
			percent: 0.5,
			some_other_field: 42,
		};
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
}
