// stole the tests archiecture from dtolnay's workshop.

use v_utils_macros::graphemics;

fn main() {
	let result = graphemics!("HELLO WORLD");
	eprintln!("{:?}", result);
}
