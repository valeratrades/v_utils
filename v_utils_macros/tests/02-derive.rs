use v_utils_macros::FromCompactFormat;

#[derive(FromCompactFormat)]
pub struct Test {
	a: i32,
	b: i32,
}

fn main() {}
