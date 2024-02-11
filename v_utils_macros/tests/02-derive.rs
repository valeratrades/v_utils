use v_utils_macros::FromVecString;

#[derive(FromVecString, Debug, Clone)]
pub struct Protocols {
	pub trailing_stop: Option<f64>,
	pub take_profit_stop_loss: Option<f64>,
	pub leading_crosses: Option<f64>,
}

fn main() {
	let _ = Protocols::try_from(vec!["0.1".to_owned(), "0.2".to_owned(), "0.3".to_owned()]);
	let _ = Protocols::try_from(vec!["0.1", "0.2", "0.3"]);
}
