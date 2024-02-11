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

/// Put on a struct with optional fields, each of which implements FromStr
///BUG: may write to the wrong field, if any of the child structs share the same acronym and same fields. In reality, shouldn't happen.
#[proc_macro_derive(FromVecString)]
pub fn derive(input: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;
	let fields = if let syn::Data::Struct(syn::DataStruct {
		fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
		..
	}) = ast.data
	{
		named
	} else {
		unimplemented!()
	};

	let init_nones = fields.iter().map(|f| {
		let ident = &f.ident;
		let ty = &f.ty;
		quote! {
			let mut #ident: #ty = None;
		}
	});

	let write_fields = fields.iter().map(|f| {
		let ident = &f.ident;
		quote! {
			#ident,
		}
	});

	let conversions = fields.iter().map(|field| {
		let field_name = &field.ident;
		let field_type = match &field.ty {
			syn::Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "Option" => {
				let generic_arg = match type_path.path.segments.last().unwrap().arguments {
					syn::PathArguments::AngleBracketed(ref args) => &args.args[0],
					_ => panic!("Expected generic argument for Option"),
				};
				quote! { #generic_arg }
			}
			_ => panic!("All fields must be of type Option<T>"),
		};

		quote! {
			if #field_name.is_none() {
				if let std::result::Result::Ok(value) = s.as_ref().to_str().unwrap_or("").parse::<#field_type>() {
					#field_name = core::option::Option::Some(value);
					continue;
				}
			}
		}
	});

	let expanded = quote! {
		impl<S: AsRef<std::ffi::OsStr>> TryFrom<Vec<S>> for #name {
			type Error = &'static str;

			fn try_from(strings: Vec<S>) -> core::result::Result<Self, Self::Error> {
				#(#init_nones)*

				for s in strings {
					#(#conversions)*

					return std::result::Result::Err("Could not parse string");
				}

				std::result::Result::Ok(#name {
					#(#write_fields)*
				})
			}
		}
	};

	expanded.into()
}
