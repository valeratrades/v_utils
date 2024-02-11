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

	let acronym = split_caps.iter().map(|s| s.chars().next().unwrap()).collect::<String>().to_lowercase();
	let acronym_caps = acronym.to_uppercase();
	let same_lower = s.to_lowercase();
	let same_upper = s.to_uppercase();
	let same = s.clone();
	let snake_case = split_caps.iter().map(|s| s.to_lowercase()).collect::<Vec<String>>().join("_");

	let unique_set = vec![acronym, acronym_caps, same_lower, same_upper, same, snake_case]
		.into_iter()
		.collect::<std::collections::HashSet<String>>();
	let unique_vec = unique_set.into_iter().collect::<Vec<String>>();

	let expanded = quote! {
		{
			let mut result: Vec<&'static str> = Vec::new();
			#(
				result.push(#unique_vec);
			)*
			result
		}
	};

	TokenStream::from(expanded)
}

/////BUG: will not work if any of the child structs share the same acronym.
//// must end with 's'

#[proc_macro_derive(FromCompactFormat)]
pub fn derive(input: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;

	let expanded = quote! {
		//impl From<Vec<String>> for #name {
		//	fn from(v: Vec<String>) -> Self {
		//		#name {
		//			#(
		//				#name::#name_variant: v[#index].parse().unwrap(),
		//			)*
		//		}
		//	}
		//}
	};
	eprintln!("Hello, {}!", stringify!(#name));

	expanded.into()
}

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
