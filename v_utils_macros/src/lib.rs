extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// returns Vec<String> of the ways to refer to a struct name
#[proc_macro]
pub fn graphemics(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::Ident);

	let s = input.to_string();
	let mut split_caps = Vec::new();
	let mut current_word = String::new();
	for c in s.chars() {
		if c.is_uppercase() && !current_word.is_empty() {
			split_caps.push(current_word);
			current_word = String::new();
		}
		current_word.push(c);
	}
	if !current_word.is_empty() {
		split_caps.push(current_word);
	}

	let accronym = split_caps.iter().map(|s| s.chars().next().unwrap()).collect::<String>().to_lowercase();
	let same_lower = s.to_lowercase();
	let same_upper = s.to_uppercase();
	let same = s.clone();
	let snake_case = split_caps.iter().map(|s| s.to_lowercase()).collect::<Vec<String>>().join("_");

	let expanded = quote! {
		{
			let result: Vec<&'static str> = vec![#accronym, #same_lower, #same_upper, #same, #snake_case];
			result
		}
	};

	TokenStream::from(expanded)
}

/////BUG: will not work if any of the child structs share the same accronym.
//// must end with 's'

//? derive what? I need it to be able to deserialize from Vec<String>
//#[proc_macro_derive()]
//pub fn derive(input: TokenStream) -> TokenStream {
//	let ast = parse_macro_input!(input as syn::DeriveInput);
//	let name = &ast.ident;
//
//	let expanded = quote! {
//		impl From<&str> for #name {
//			fn from(s: &str) -> Self {
//				let mut split = s.split_whitespace();
//				#name {
//					#(
//						#name::#name_variant: split.next().unwrap().parse().unwrap(),
//					)*
//				}
//			}
//		}
//	};
//
//	expanded.into()
//}
