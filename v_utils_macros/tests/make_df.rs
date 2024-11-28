use v_utils_macros::make_df;
use insta::assert_snapshot;


fn main() {
	let expected_code = r#"
				let mut open_time = Vec::new();
				let mut close = Vec::new();
				let mut volume = Vec::new();

				for kline in json {
						if let Some(open_time) = kline.get(0) {
								open_time.push(open_time.as_str().unwrap().parse::<i64>().unwrap());
						}
						if let Some(close) = kline.get(4) {
								close.push(close.as_str().unwrap().parse::<f64>().unwrap());
						}
						if let Some(volume) = kline.get(5) {
								volume.push(volume.as_str().unwrap().parse::<f64>().unwrap());
						}
				}

				let df = df![
						"open_time" => open_time,
						"close" => close,
						"volume" => volume
				]
				.expect("Failed to create DataFrame");
				df
				"#;

	let df = make_df![
		json =>
		(0, i64, open_time)
		(4, f64, close)
		(5, f64, volume)
	];
	assert_snapshot!(df, @r###""###);
}
