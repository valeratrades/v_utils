extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn to_lowercase(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as LitStr);
	let s = input.value().to_lowercase();
	let expanded = quote! {
		{
			let result: &'static str = #s;
			result
		}
	};
	TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_lowercase_macro() {
		// You need to execute the resulting Rust code from the macro to test its output
		let result: &'static str = to_lowercase!("HELLO WORLD");
		assert_eq!(result, "hello world");
	}
}

/////BUG: will not work if any of the child structs share the same accronym.
//// must end with 's'
////- umbrella_compact_optional!(Protocol, [SAR, TrailingStop, TpSl, LeadingCrosses]);
//// and then if need a wrapper
////- umbrella_compact_optional_wrapped!(Protocol, ProtocolWrapper::new([SAR, TrailingStop, TpSl, LeadingCrosses]);
////TODO!: assert that first split on "::" is followed by "new("
////TODO!!!!!!!!!!!!!: implement Umbrella struct constructor

//#[proc_macro]
//pub fn pascal_to_snake(input: TokenStream) -> TokenStream {
//	let input = parse_macro_input!(input as DeriveInput);
//	let name = input.ident;
//	let snake_case_name = to_snake_case(&name.to_string());
//	let snake_case_ident = Ident::new(&snake_case_name, name.span());
//	TokenStream::from(snake_case_ident)
//}
//
//fn to_snake_case(s: &str) -> String {
//	let mut snake_case = String::new();
//	for (i, char) in s.chars().enumerate() {
//		if char.is_uppercase() && i != 0 {
//			snake_case.push('_');
//		}
//		snake_case.push(char.to_lowercase().next().unwrap());
//	}
//	snake_case
//}
//
//#[macro_export]
//macro_rules! umbrella_compact_optional {
//	($name:ident, [ $struct: ty, * ]) => {
//#[derive(Clone, Debug)]
//pub enum concat_idents!($name, s) {
//	$(
//		pascal_to_snake_case!($struct): $struct,
//	)*
//}
//};}
////- umbrella_compact_optional!(Protocol, [SAR, TrailingStop, TpSl, LeadingCrosses]);
//
//#[cfg(test)]
//mod tests {
//	use super::*;
//	use proc_macro::TokenStream;
//	use quote::quote;
//
//	#[test]
//	fn test_pascal_to_snake() {
//		let input = TokenStream::from(quote! {
//			struct TestStruct;
//		});
//
//		let expected_output = "test_struct".to_string();
//		let actual_output = pascal_to_snake(input).to_string();
//
//		assert!(
//			actual_output.contains(&expected_output),
//			"Expected '{}', found '{}'",
//			expected_output,
//			actual_output
//		);
//	}
//}
