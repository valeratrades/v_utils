use v_utils_macros::graphemics;

macro_rules! init_compact_format {
	($name:ident) => {
		println!("{}: {:?}", stringify!($name), graphemics!($name));
	};
}

fn main() {
	init_compact_format!(TestStruct);
}
