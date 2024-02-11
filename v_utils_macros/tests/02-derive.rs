use v_utils_macros::FromVecString;

#[derive(FromVecString)]
pub struct Test {
	a: i32,
	b: i32,
}

fn main() {}
