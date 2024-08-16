use v_utils_macros::graphemics;

fn main() {
	let mut gr = graphemics!(SAR);
	//assert!(gr.len() == 3);
	//assert!(gr.contains(&"SAR"));
	//assert!(gr.contains(&"sar"));
	//assert!(gr.contains(&"s_a_r"));
	gr.sort();
	insta::assert_debug_snapshot!(gr, @r###"
 [
     "SAR",
     "s_a_r",
     "sar",
 ]
 "###);

	let mut gr = graphemics!(TakeProfitStopLoss);
	gr.sort();
	insta::assert_debug_snapshot!(gr, @r###"
 [
     "TAKEPROFITSTOPLOSS",
     "TPSL",
     "TakeProfitStopLoss",
     "take_profit_stop_loss",
     "takeprofitstoploss",
     "tpsl",
 ]
 "###);

	let mut gr = graphemics!(Oneword);
	gr.sort();
	insta::assert_debug_snapshot!(gr, @r###"
 [
     "ONEWORD",
     "Oneword",
     "oneword",
 ]
 "###);
}
