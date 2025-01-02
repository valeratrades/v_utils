#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input, token, Data, DeriveInput, Ident, LitInt, Token,
};

/// returns `Vec<String>` of the ways to refer to a struct name
///
/// For some reason not a `HashSet`, no clue why.
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

	let graphems = [acronym, acronym_caps, same_lower, same_upper, same, snake_case];
	let mut unique_items = Vec::new();
	for item in graphems.into_iter() {
		if !unique_items.contains(&item) {
			unique_items.push(item);
		}
	}
	let unique_items_valid = unique_items.into_iter().filter(|s| s.len() != 1).collect::<Vec<String>>();

	let expanded = quote! {
		{
			let mut result: Vec<&'static str> = Vec::new();
			#(
			result.push(#unique_items_valid);
		)*
			result
		}
	};

	TokenStream::from(expanded)
}

/// cli-like string serialization format, with focus on compactness
///
/// A brain-dead child format of mine. Idea is to make parameter specification as compact as possible. Very similar to how you would pass arguments to `clap`, but here all the args are [arg(short)] by default, and instead of spaces, equal signs, and separating names from values, we write `named_argument: my_value` as `-nmy_value`. Entries are separated by ':' char.
///
/// Macro generates FromStr and Display; assuming this format.
///```rust
///#[cfg(feature = "macros")] {
///#[cfg(feature = "trades")] {
///use v_utils::macros::CompactFormat;
///use v_utils::trades::{Timeframe, TimeframeDesignator};
///
///#[derive(CompactFormat, PartialEq, Debug)]
///pub struct SAR {
///	 pub start: f64,
///	 pub increment: f64,
///	 pub max: f64,
///	 pub timeframe: Timeframe,
///}
///
///let sar = SAR { start: 0.07, increment: 0.02, max: 0.15, timeframe: Timeframe { designator: TimeframeDesignator::Minutes, n: 5 } };
///let params_str = "sar:s0.07:i0.02:m0.15:t5m";
///assert_eq!(sar, params_str.parse::<SAR>().unwrap());
///let sar_write = sar.to_string();
///assert_eq!(params_str, sar_write);
///}
///}
///```
#[proc_macro_derive(CompactFormat)]
pub fn derive_compact_format(input: TokenStream) -> TokenStream {
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

	let mut first_chars: Vec<char> = Vec::new();
	for field in fields {
		let first_char = field.ident.as_ref().unwrap().to_string().chars().next().unwrap();
		if !first_chars.contains(&first_char) {
			first_chars.push(first_char);
		} else {
			panic!("Field names must be unique");
		}
	}

	let n_fields = fields.len();

	let map_fields_to_chars = fields.iter().map(|f| {
		let ident = &f.ident;
		let ty = &f.ty;
		let first_char = ident.as_ref().unwrap().to_string().chars().next().unwrap();
		quote! {
			#ident: provided_params.get(&#first_char).unwrap().parse::<#ty>()?,
		}
	});

	let display_fields = fields.iter().map(|f| {
		let ident = &f.ident;
		let first_char = ident.as_ref().unwrap().to_string().chars().next().unwrap();
		quote! {
			write!(f, ":{}{}", #first_char, self.#ident)?;
		}
	});

	let expanded = quote! {
		impl std::str::FromStr for #name {
			type Err = v_utils::__internal::eyre::Report;

			fn from_str(s: &str) -> v_utils::__internal::eyre::Result<Self> {
				let (name, params_part) = s.split_once(':').unwrap_or((s, ""));
				let params_split = if (params_part == "" || params_part == "_" ) { Vec::new() } else { params_part.split(':').collect::<Vec<&str>>() };
				if params_split.len() != #n_fields {
					v_utils::__internal::eyre::bail!("Expected {} fields, got {}", #n_fields, params_split.len());
				}
				let graphemics = v_utils::macros::graphemics!(#name);
				if !graphemics.contains(&name) {
					v_utils::__internal::eyre::bail!("Incorrect name provided. Expected one of: {:?}", graphemics);
				}

				let mut provided_params: std::collections::HashMap<char, &str> = std::collections::HashMap::new();
				for param in params_split {
					if let Some(first_char) = param.chars().next() {
						let value = &param[1..];
						provided_params.insert(first_char, value);
					}
				}
				Ok(#name {
					#(#map_fields_to_chars)*
			})
			}
		}

		impl std::fmt::Display for #name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				let graphemics = v_utils::macros::graphemics!(#name);
				let a_struct_name = graphemics[0];
				write!(f, "{}", a_struct_name)?;

				#(#display_fields)*

				std::result::Result::Ok(())
			}
		}
	};

	expanded.into()
}

/// Put on a struct with optional fields, each of which implements FromStr
///
///BUG: may write to the wrong field, if any of the child structs share the same acronym AND same fields. In reality, shouldn't happen.
#[proc_macro_derive(OptionalFieldsFromVecStr)]
pub fn derive_optional_fields_from_vec_str(input: TokenStream) -> TokenStream {
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
				if let std::result::Result::Ok(value) = s.as_ref().parse::<#field_type>() {
					#field_name = core::option::Option::Some(value);
					continue;
				}
			}
		}
	});

	let expanded = quote! {
	impl<S: AsRef<str>> TryFrom<Vec<S>> for #name {
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

#[proc_macro_derive(VecFieldsFromVecStr)]
pub fn derive_optioinal_vec_fields_from_vec_str(input: TokenStream) -> TokenStream {
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

	let init_empty_vecs = fields.iter().map(|f| {
		let ident = &f.ident;
		let ty = &f.ty;
		quote! {
			let mut #ident: #ty = Vec::new();
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
			syn::Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "Vec" => {
				let generic_arg = match type_path.path.segments.last().unwrap().arguments {
					syn::PathArguments::AngleBracketed(ref args) => &args.args[0],
					_ => panic!("Expected generic argument for Vec"),
				};
				quote! { #generic_arg }
			}
			_ => panic!("All fields must be of type Vec<T>"),
		};

		quote! {
			if let std::result::Result::Ok(value) = s.as_ref().parse::<#field_type>() {
				#field_name.push(value);
				continue;
			}
		}
	});

	let expanded = quote! {
	impl<S: AsRef<str>> TryFrom<Vec<S>> for #name {
		type Error = &'static str;

		fn try_from(strings: Vec<S>) -> core::result::Result<Self, Self::Error> {
			#(#init_empty_vecs)*

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

#[proc_macro_derive(MyConfigPrimitives)]
pub fn deserialize_with_private_values(input: TokenStream) -> TokenStream {
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

	let (helper_fields, init_fields): (Vec<_>, Vec<_>) = fields
		.iter()
		.map(|f| {
			let ident = &f.ident;
			let ty = &f.ty;
			let type_string = quote! { #ty }.to_string();

			match type_string.as_str() {
				"String" => (
					quote! { #ident: PrivateValue },
					quote! { #ident: helper.#ident.into_string().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to convert {} to string: {}", stringify!(#ident), e)))? },
				),
				"PathBuf" => (quote! { #ident: v_utils::io::ExpandedPath }, quote! { #ident: helper.#ident.0 }),
				_ => (quote! { #ident: #ty }, quote! { #ident: helper.#ident }),
			}
		})
		.unzip();

	let gen = quote! {
		impl<'de> v_utils::__internal::serde::Deserialize<'de> for #name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
				D: v_utils::__internal::serde::de::Deserializer<'de>,
			{
				use v_utils::__internal::eyre::WrapErr;

				#[derive(Clone, Debug)]
				enum PrivateValue {
					String(String),
					Env { env: String },
				}
				impl PrivateValue {
					pub fn into_string(&self) -> v_utils::__internal::eyre::Result<String> {
						match self {
							PrivateValue::String(s) => Ok(s.clone()),
							PrivateValue::Env { env } => std::env::var(env).wrap_err_with(|| format!("Environment variable '{}' not found", env)),
						}
					}
				}
				impl<'de> v_utils::__internal::serde::Deserialize<'de> for PrivateValue {
					fn deserialize<D>(deserializer: D) -> Result<PrivateValue, D::Error>
				where
						D: v_utils::__internal::serde::de::Deserializer<'de>,
					{
						struct PrivateValueVisitor;

						impl<'de> v_utils::__internal::serde::de::Visitor<'de> for PrivateValueVisitor {
							type Value = PrivateValue;

							fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
								formatter.write_str("a string or a map with a single key 'env'")
							}

							fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
						where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::String(value.to_owned()))
							}

							fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
						where
								M: v_utils::__internal::serde::de::MapAccess<'de>,
							{
								let key: String = access.next_key()?.ok_or_else(|| v_utils::__internal::serde::de::Error::custom("expected a key"))?;
								if key == "env" {
									let value: String = access.next_value()?;
									Ok(PrivateValue::Env { env: value })
								} else {
									Err(v_utils::__internal::serde::de::Error::custom("expected key to be 'env'"))
								}
							}
						}

						deserializer.deserialize_any(PrivateValueVisitor)
					}
				}


				#[derive(v_utils::__internal::serde::Deserialize)]
				#[serde(crate = "v_utils::__internal::serde")]
				struct Helper {
					#(#helper_fields),*
				}
				let helper = Helper::deserialize(deserializer)?;

				Ok(#name {
					#(#init_fields),*
				})
			}
		}
	};

	gen.into()
}

// make_df! {{{
/// Structure to hold the entire macro input
struct Field {
	_parens: token::Paren,
	index: LitInt,
	_comma1: Token![,],
	dtype: Ident,
	_comma2: Token![,],
	name: Ident,
}
impl Parse for Field {
	fn parse(input: ParseStream) -> Result<Self, syn::Error> {
		let content;
		Ok(Field {
			_parens: syn::parenthesized!(content in input),
			index: content.parse()?,
			_comma1: content.parse()?,
			dtype: content.parse()?,
			_comma2: content.parse()?,
			name: content.parse()?,
		})
	}
}

/// Structure to hold the entire macro input
struct DataFrameDef {
	values_vec: Ident,
	_arrow: Token![=>],
	fields: Vec<Field>,
}
impl Parse for DataFrameDef {
	fn parse(input: ParseStream) -> Result<Self, syn::Error> {
		let values_vec: Ident = input.parse()?;
		let _arrow: Token![=>] = input.parse()?;

		let mut fields = Vec::new();
		//TODO!: add optional comma at the end
		while !input.is_empty() {
			fields.push(input.parse()?);
		}

		Ok(DataFrameDef { values_vec, _arrow, fields })
	}
}

#[proc_macro]
pub fn make_df(input: TokenStream) -> TokenStream {
	let DataFrameDef { values_vec, fields, .. } = parse_macro_input!(input as DataFrameDef);

	fn vec_name(name: &Ident) -> Ident {
		let vec_name = format!("{name}s");
		syn::Ident::new(&vec_name, name.span())
	}

	let vec_declarations = fields.iter().map(|field| {
		let vec_ident = vec_name(&field.name);
		let ty = &field.dtype;
		quote! {
			let mut #vec_ident: Vec<#ty> = Vec::new();
		}
	});

	// Generate tuple of indices for pattern matching
	let indices = fields.iter().map(|field| {
		let idx = &field.index;
		quote! {
			value.get(#idx)
		}
	});

	// Generate push statements with appropriate type conversion
	let push_statements = fields.iter().map(|field| {
		let name = &field.name;
		let vec_name = vec_name(name);
		let dtype = &field.dtype;

		// might not cover all the polars as methods correctly, may need to be updated to an explicit match statement through them.
		let as_method = syn::Ident::new(&format!("as_{dtype}"), dtype.span());

		quote! {
			// inefficient but that's data-analysis, don't think I care
			// unwrap_or would be preferrable as we expect this line to be taken ~50% of the time, but it breaks expected type (maybe it's still possible - try again later)
			#vec_name.push(#name.#as_method().unwrap_or_else(|| #name.as_str().unwrap().parse::<#dtype>().unwrap()));
		}
	});

	let df_fields = fields.iter().map(|field| {
		let vec_name = vec_name(&field.name);
		let name_str = &field.name.to_string();
		quote! {
			#name_str => #vec_name
		}
	});

	// pretty sure there is a better way to do this, but eh
	let temp_vars = fields
		.iter()
		.map(|field| {
			let name = format!("{}", field.name);
			syn::Ident::new(&name, proc_macro2::Span::call_site())
		})
		.collect::<Vec<_>>();

	quote! {
	{
		#(#vec_declarations)*

		for value in #values_vec {
			if let (#(Some(#temp_vars)),*) = (#(#indices),*) {
				#(#push_statements;)*
			}
			}

			let df = polars::df![
			#(#df_fields),*
		].expect("Failed to create DataFrame");
			df
		}
	}
	.into()
}
//,}}}

#[proc_macro_derive(WrapNew)]
pub fn wrap_new(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	let inner_type = match &input.data {
		Data::Struct(data) => match &data.fields {
			syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed.first().unwrap().ty,
			_ => panic!("NewWrapper can only be derived for tuple structs with one field"),
		},
		_ => panic!("NewWrapper can only be derived for tuple structs"),
	};

	let expanded = quote! {
		impl #name {
			pub fn new() -> Self {
				Self(<#inner_type>::new())
			}
		}
	};

	TokenStream::from(expanded)
}

//TODO!!!: finish and test
#[proc_macro_derive(ScreamIt)]
pub fn scream_it(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;

	// Ensure it's an enum
	let variants = if let Data::Enum(syn::DataEnum { variants, .. }) = &input.data {
		variants
	} else {
		panic!("#[derive(ScreamIt)] can only be used on enums");
	};

	// Generate the Display implementation
	let display_impl = {
		let arms = variants.iter().map(|variant| {
			let variant_name = &variant.ident;
			let screamed_name = variant_name.to_string().to_uppercase();
			quote! {
				Self::#variant_name => write!(f, #screamed_name),
			}
		});

		quote! {
			impl std::fmt::Display for #name {
				fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
					match self {
						#(#arms)*
					}
				}
			}
		}
	};

	// Generate the FromStr implementation
	let from_str_impl = {
		let arms = variants.iter().map(|variant| {
			let variant_name = &variant.ident;
			let screamed_name = variant_name.to_string().to_uppercase();
			quote! {
				#screamed_name => Ok(Self::#variant_name),
			}
		});

		quote! {
			impl std::str::FromStr for #name {
				type Err = ();

				fn from_str(s: &str) -> Result<Self, Self::Err> {
					match s {
						#(#arms)*
						_ => Err(()),
					}
				}
			}
		}
	};

	let expanded = quote! {
		#display_impl
		#from_str_impl
	};

	TokenStream::from(expanded)
}
