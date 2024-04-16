use v_utils_macros::{CompactFormat, OptionalFieldsFromVecStr, VecFieldsFromVecStr};

#[derive(CompactFormat, Debug, Clone, PartialEq)]
struct TrailingStop {
	percent: f64,
}

#[derive(OptionalFieldsFromVecStr, Debug, Clone, PartialEq)]
struct OptionalProtocols {
	trailing_stop: Option<TrailingStop>,
	take_profit_stop_loss: Option<f64>,
	leading_crosses: Option<f64>,
}

#[derive(VecFieldsFromVecStr, Debug, Clone, PartialEq)]
struct VecProtocols {
	trailing_stop: Vec<TrailingStop>,
	take_profit_stop_loss: Vec<f64>,
	leading_crosses: Vec<f64>,
}

fn main() {
	let o1 = OptionalProtocols::try_from(vec!["0.1", "ts:p-0.2", "0.3"]).unwrap();
	assert_eq!(
		o1,
		OptionalProtocols {
			trailing_stop: Some(TrailingStop { percent: -0.2 }),
			take_profit_stop_loss: Some(0.1),
			leading_crosses: Some(0.3)
		}
	);

	let o2 = OptionalProtocols::try_from(vec!["0.2".to_owned(), "0.3".to_owned()]).unwrap();
	assert_eq!(
		o2,
		OptionalProtocols {
			trailing_stop: None,
			take_profit_stop_loss: Some(0.2),
			leading_crosses: Some(0.3)
		}
	);

	let v1 = VecProtocols::try_from(vec!["ts:p0.2", "ts:p-0.2"]).unwrap();
	assert_eq!(
		v1,
		VecProtocols {
			trailing_stop: vec![TrailingStop { percent: 0.2 }, TrailingStop { percent: -0.2 }],
			take_profit_stop_loss: Vec::new(),
			leading_crosses: Vec::new()
		}
	);
}
