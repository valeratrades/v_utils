use v_utils_macros::graphemics;

fn main() {
	let gr = graphemics!(SAR);
	assert!(gr.len() == 3);
	assert!(gr.contains(&"SAR"));
	assert!(gr.contains(&"sar"));
	assert!(gr.contains(&"s_a_r"));

	let gr = graphemics!(TakeProfitStopLoss);
	assert!(gr.len() == 6);
	assert!(gr.contains(&"TPSL"));
	assert!(gr.contains(&"tpsl"));
	assert!(gr.contains(&"take_profit_stop_loss"));
	assert!(gr.contains(&"takeprofitstoploss"));
	assert!(gr.contains(&"TAKEPROFITSTOPLOSS"));
	assert!(gr.contains(&"TakeProfitStopLoss"));
}
