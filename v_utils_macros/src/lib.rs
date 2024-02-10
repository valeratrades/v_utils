extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// returns Vec<String> of the ways to refer to a struct name
#[proc_macro]
pub fn graphemics(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as LitStr);

	let s = input.value();
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
