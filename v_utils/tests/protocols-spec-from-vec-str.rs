use v_utils::init_compact_format;
use v_utils::trades::Timeframe;
use v_utils_macros::FromVecStr;

init_compact_format!(SAR, [(start, f64), (increment, f64), (max, f64), (timeframe, Timeframe)]);
init_compact_format!(TrailingStop, [(percent, f64)]);
init_compact_format!(TPSL, [(tp, f64), (sl, f64)]);
init_compact_format!(LeadingCrosses, [(symbol, String), (price, f64)]);

#[derive(Debug, FromVecStr)]
pub struct Protocols {
	pub trailing_stop: Option<TrailingStop>,
	pub sar: Option<SAR>,
	pub tpsl: Option<TPSL>,
	pub leading_crosses: Option<LeadingCrosses>,
}

fn main() {
	let protocols = Protocols::try_from(vec!["ts-p0.5"]).unwrap();
	assert_eq!(protocols.trailing_stop.unwrap().percent, 0.5);
	assert_eq!(protocols.sar, None);
	assert_eq!(protocols.tpsl, None);
	assert_eq!(protocols.leading_crosses, None);
}
