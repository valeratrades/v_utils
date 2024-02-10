// stole the tests archiecture from dtolnay's workshop.

use v_utils_macros::graphemics;

fn main() {
	let result: Vec<String> = graphemics!("HELLO WORLD");
	eprintln!("{:?}", result);
}
