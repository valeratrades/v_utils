#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
extern crate proc_macro2;
use heck::{AsShoutySnakeCase, AsSnakeCase};
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
	Data, DeriveInput, Fields, Ident, LitInt, Token,
	parse::{Parse, ParseStream},
	parse_macro_input, token,
};

// helpers {{{
/// A helper function to know location of errors in `quote!{}`s
fn _dbg_token_stream(expanded: proc_macro2::TokenStream, name: &str) -> proc_macro2::TokenStream {
	let fpath = format!("/tmp/{}_expanded/{name}.rs", env!("CARGO_PKG_NAME"));
	std::fs::create_dir_all(std::path::PathBuf::from(&fpath).parent().unwrap()).unwrap();
	std::fs::write(&fpath, expanded.to_string()).unwrap();
	std::process::Command::new("rustfmt").arg("--edition=2024").arg(&fpath).output().unwrap();
	quote! {include!(#fpath); }
}
macro_rules! _dbg_tree {
	($target:expr) => {
		let fpath = concat!("/tmp/", env!("CARGO_PKG_NAME"), "_dbg.rs");
		let dbg_str = format!("{:#?}", $target);
		std::fs::write(fpath, dbg_str).unwrap();
	};
}
fn is_option_type(type_path: &syn::TypePath) -> bool {
	if let Some(segment) = type_path.path.segments.last() {
		return segment.ident == "Option";
	}
	false
}

// Helper function to check if a type path is Vec<T>
fn is_vec_type(type_path: &syn::TypePath) -> bool {
	if let Some(segment) = type_path.path.segments.last() {
		return segment.ident == "Vec";
	}
	false
}

// Helper function to check if a type is a specific primitive
fn is_type(type_path: &syn::TypePath, type_name: &str) -> bool {
	if let Some(segment) = type_path.path.segments.last() {
		return segment.ident == type_name;
	}
	false
}

// Extract inner type from Option<T>
fn extract_option_inner_type(type_path: &syn::TypePath) -> &syn::Type {
	if let Some(segment) = type_path.path.segments.last() {
		if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
			if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
				return inner_type;
			}
		}
	}
	// Return a dummy type if extraction fails
	panic!("Failed to extract inner type from Option")
}

// Extract inner type from Vec<T>
fn extract_vec_inner_type(type_path: &syn::TypePath) -> &syn::Type {
	if let Some(segment) = type_path.path.segments.last() {
		if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
			if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
				return inner_type;
			}
		}
	}
	// Return a dummy type if extraction fails
	panic!("Failed to extract inner type from Vec")
}
fn _single_option_wrapped_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
	match ty {
		syn::Type::Path(type_path) =>
			if type_path.path.segments.last().unwrap().ident == "Option" {
				quote! { #ty }
			} else {
				quote! { Option<#ty> }
			},
		_ => quote! { Option<#ty> },
	}
}

//,}}}

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
///#[derive(CompactFormat, Debug, PartialEq)]
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
	let ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;
	let fields = if let Data::Struct(syn::DataStruct {
		fields: Fields::Named(syn::FieldsNamed { ref named, .. }),
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
	let ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;
	let fields = if let Data::Struct(syn::DataStruct {
		fields: Fields::Named(syn::FieldsNamed { ref named, .. }),
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

#[proc_macro_derive(MyConfigPrimitives, attributes(private_value, serde, settings, primitives))]
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

			// Check if field has #[private_value] attribute
			let has_private_value_attr = f.attrs.iter().any(|attr| {
				attr.path().is_ident("private_value")
			});

			// Check if field has #[primitives(skip)] attribute
			let has_primitives_skip_attr = f.attrs.iter().any(|attr| {
				if attr.path().is_ident("primitives") {
					if let Ok(nested) = attr.parse_args::<syn::Ident>() {
						return nested == "skip";
					}
				}
				false
			});

			// Collect all attributes except private_value, primitives, and settings to forward to Helper
			let forwarded_attrs = f.attrs.iter().filter(|attr| {
				!attr.path().is_ident("private_value") && !attr.path().is_ident("settings") && !attr.path().is_ident("primitives")
			}).collect::<Vec<_>>();

			// Check if type is Option<T>
			let is_option = if let syn::Type::Path(type_path) = ty {
				is_option_type(type_path)
			} else {
				false
			};

			// If field has #[primitives(skip)], don't apply PrivateValue transformation
			if has_primitives_skip_attr {
				(quote! {
					#(#forwarded_attrs)*
					#ident: #ty
				}, quote! { #ident: helper.#ident })
			} else if has_private_value_attr {
				// For fields marked with #[private_value], wrap in PrivateValue and use FromStr
				(
					quote! {
						#(#forwarded_attrs)*
						#ident: PrivateValue
					},
					quote! { #ident: helper.#ident.into_string().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to convert {} to string: {}", stringify!(#ident), e)))?.parse().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to parse {} from string: {:?}", stringify!(#ident), e)))? },
				)
			} else if is_option {
				// Handle Option<T> types
				if let syn::Type::Path(type_path) = ty {
					let inner_type = extract_option_inner_type(type_path);
					let inner_type_string = quote! { #inner_type }.to_string();

					match inner_type_string.as_str() {
						"String" => (
							quote! {
								#(#forwarded_attrs)*
								#ident: Option<PrivateValue>
							},
							quote! {
								#ident: match helper.#ident {
									Some(pv) => Some(pv.into_string().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to convert {} to string: {}", stringify!(#ident), e)))?),
									None => None,
								}
							},
						),
						"PathBuf" => (
							quote! {
								#(#forwarded_attrs)*
								#ident: Option<v_utils::io::ExpandedPath>
							},
							quote! { #ident: helper.#ident.map(|ep| ep.0) },
						),
						"SecretString" => (
							quote! {
								#(#forwarded_attrs)*
								#ident: Option<PrivateValue>
							},
							quote! {
								#ident: match helper.#ident {
									Some(pv) => Some(secrecy::SecretString::new(pv.into_string().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to convert {} to string: {}", stringify!(#ident), e)))?.into_boxed_str())),
									None => None,
								}
							},
						),
						_ => (quote! {
							#(#forwarded_attrs)*
							#ident: #ty
						}, quote! { #ident: helper.#ident }),
					}
				} else {
					(quote! {
						#(#forwarded_attrs)*
						#ident: #ty
					}, quote! { #ident: helper.#ident })
				}
			} else {
				match type_string.as_str() {
					"String" => (
						quote! {
							#(#forwarded_attrs)*
							#ident: PrivateValue
						},
						quote! { #ident: helper.#ident.into_string().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to convert {} to string: {}", stringify!(#ident), e)))? },
					),
					"PathBuf" => (quote! {
						#(#forwarded_attrs)*
						#ident: v_utils::io::ExpandedPath
					}, quote! { #ident: helper.#ident.0 }),
					"SecretString" => (
						quote! {
							#(#forwarded_attrs)*
							#ident: PrivateValue
						},
						quote! { #ident: secrecy::SecretString::new(helper.#ident.into_string().map_err(|e| v_utils::__internal::serde::de::Error::custom(format!("Failed to convert {} to string: {}", stringify!(#ident), e)))?.into_boxed_str()) },
					),
					_ => (quote! {
						#(#forwarded_attrs)*
						#ident: #ty
					}, quote! { #ident: helper.#ident }),
				}
			}
		})
		.unzip();

	let q = quote! {
		impl<'de> v_utils::__internal::serde::Deserialize<'de> for #name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
				D: v_utils::__internal::serde::de::Deserializer<'de>,
			{
				use v_utils::__internal::eyre::WrapErr;

				#[derive(Clone, Debug)]
				enum PrivateValue {
					Direct(String),
					Env { env: String },
				}
				impl Default for PrivateValue {
					fn default() -> Self {
						PrivateValue::Direct(String::new())
					}
				}
				impl PrivateValue {
					pub fn into_string(&self) -> v_utils::__internal::eyre::Result<String> {
						match self {
							PrivateValue::Direct(s) => Ok(s.clone()),
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
								formatter.write_str("a value (string, number, bool, etc.) or a map with a single key 'env'")
							}

							fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
							where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_string()))
							}

							fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
							where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_string()))
							}

							fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
							where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_string()))
							}

							fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
							where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_string()))
							}

							fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
							where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_string()))
							}

							fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
							where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_string()))
							}

							fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
						where
								E: v_utils::__internal::serde::de::Error,
							{
								Ok(PrivateValue::Direct(value.to_owned()))
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
	q.into()
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

//BUG: doesn't convert to SCREAMING_SNAKE_CASE, but simply uppercases everything
/// Implements Display and FromStr for variants of an enum, using SCREAMING_SNAKE_CASE
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
			let screamed_name = AsShoutySnakeCase(variant_name.to_string()).to_string();
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
			let screamed_name = AsShoutySnakeCase(variant_name.to_string()).to_string();
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

	let serialize_impl = quote! {
		impl serde::Serialize for #name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				serializer.serialize_str(&self.to_string())
			}
		}
	};

	let deserialize_impl = quote! {
		impl<'de> serde::Deserialize<'de> for #name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				let s = String::deserialize(deserializer)?;
				s.parse().map_err(|_| serde::de::Error::custom("invalid enum value"))
			}
		}
	};

	let expanded = quote! {
		#display_impl
		#from_str_impl
		#serialize_impl
		#deserialize_impl
	};

	TokenStream::from(expanded)
}

// Settings {{{

//TODO!: error messages (like the one about necessity of deriving SettingsNested on children)
//NB: requires `clap` to be in the scope (wouldn't make sense to bring it with the lib, as it's meant to be used in tandem and a local import will always be necessary)
/// Derive macro for application settings that integrates config files, environment variables, and CLI flags.
///
/// # Features
/// - Loads config from multiple sources with precedence: CLI flags > Environment variables > Config file
/// - Supports multiple config formats: TOML, JSON, YAML, JSON5, RON, INI, and Nix
/// - Automatically searches for config files in XDG-compliant directories
/// - Generates `SettingsFlags` struct for CLI integration with clap
/// - Nix config files are evaluated using `nix eval --json --impure` and must return a valid attribute set
///
/// # Config file resolution
/// 1. If `--config` flag is provided, uses that file (supports .nix extension)
/// 2. Otherwise, checks for `~/.config/<app_name>.nix` first
/// 3. Falls back to searching for other formats in:
///    - `~/.config/<app_name>.{toml,json,yaml,json5,ron,ini}`
///    - `~/.config/<app_name>/config.{toml,json,yaml,json5,ron,ini}`
///
/// # Example
/// ```ignore
/// #[derive(MyConfigPrimitives, Settings)]
/// pub struct AppConfig {
///     pub host: String,
///     pub port: u16,
/// }
/// ```
#[cfg(feature = "cli")]
#[proc_macro_derive(Settings, attributes(settings))]
pub fn derive_setings(input: TokenStream) -> proc_macro::TokenStream {
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

	//DEPRECATE: split over xdg (previously was there for interop with clients targetting WASM)
	//#[cfg(feature = "xdg")]
	let xdg_conf_dir = quote_spanned! { proc_macro2::Span::call_site()=>
		let xdg_dirs = ::v_utils::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")); //HACK: should use a method from `v_utils::io`, where use of `xdg` is conditional on an unrelated feature. Hardcoding `xdg` here problematic.
		let xdg_conf_dir = xdg_dirs.get_config_home().unwrap().parent().unwrap().display().to_string();
	};
	//#[cfg(not(feature = "xdg"))]
	//let xdg_conf_dir = quote_spanned! { proc_macro2::Span::call_site()=>
	//	let xdg_conf_dir = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap()));
	//};

	// Generate field lists for validation (exclude skipped fields)
	let all_field_names: Vec<_> = fields
		.iter()
		.filter_map(|f| {
			// Check if field has #[settings(skip)]
			let has_skip_attr = f.attrs.iter().any(|attr| {
				if attr.path().is_ident("settings") {
					if let Ok(nested) = attr.parse_args::<syn::Ident>() {
						return nested == "skip";
					}
				}
				false
			});

			if has_skip_attr { None } else { Some(f.ident.as_ref().unwrap().to_string()) }
		})
		.collect();

	let field_name_strings = all_field_names.iter().map(|name| quote! { #name });

	let try_build = quote_spanned! {name.span()=>
		//#[cfg(not(feature = "hydrate"))]
		impl #name {
			///NB: must have `Cli` struct in the same scope, with clap derived, and `insert_clap_settings!()` macro having had been expanded inside it.
			pub fn try_build(flags: SettingsFlags) -> Result<Self, ::v_utils::__internal::eyre::Report> {
				let path = flags.config.as_ref().map(|p| p.0.clone());
				let app_name = env!("CARGO_PKG_NAME");

				#xdg_conf_dir

				let location_bases = [
					format!("{xdg_conf_dir}/{app_name}"),
					format!("{xdg_conf_dir}/{app_name}/config"), //
				];
				let supported_exts = ["toml", "json", "yaml", "json5", "ron", "ini"];
				let locations: Vec<std::path::PathBuf> = location_bases.iter().flat_map(|base| supported_exts.iter().map(move |ext| std::path::PathBuf::from(format!("{base}.{ext}")))).collect();

				let mut builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::Environment::with_prefix(app_name).separator("__"/*default separator is '.', which I don't like being present in var names*/)).add_source(flags);

				let mut err_msg = "Could not construct v_utils::__internal::config from aggregated sources (conf, env, flags, cache).".to_owned();
				use ::v_utils::__internal::eyre::WrapErr as _; //HACK: problematic as could be re-exporting
				let (raw, file_config): (::v_utils::__internal::config::Config, Option<::v_utils::__internal::config::Config>) = match path {
					Some(path) => {
						// Check if it's a .nix file
						if path.to_str().map(|s| s.ends_with(".nix")).unwrap_or(false) {
							let json_str = Self::eval_nix_file(path.to_str().unwrap())?;
							let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
							let file_only = file_builder.clone().build().ok();
							let builder = builder.add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
							(builder.build()?, file_only)
						} else {
							let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from(path.clone()).required(true));
							let file_only = file_builder.clone().build().ok();
							let builder = builder.add_source(::v_utils::__internal::config::File::from(path.clone()).required(true));
							(builder.build()?, file_only)
						}
					}
					None => {
						// Check for .nix file first
						let nix_path = format!("{xdg_conf_dir}/{app_name}.nix");
						if std::path::Path::new(&nix_path).exists() {
							let json_str = Self::eval_nix_file(&nix_path)?;
							let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
							let file_only = file_builder.clone().build().ok();
							let builder = builder.add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
							(builder.build()?, file_only)
						} else {
							let mut conf_files_found = Vec::new();
							for location in locations.iter() {
								if location.exists() {
									conf_files_found.push(location);
								}
							}
							let file_only = match conf_files_found.len() {
								0 => {
									err_msg.push_str(&format!("\nNOTE: conf file is missing. Searched in {:?}", locations));
									None
								},
								1 => {
									let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from(conf_files_found[0].as_path()).required(true));
									builder = builder.add_source(::v_utils::__internal::config::File::from(conf_files_found[0].as_path()).required(true));
									file_builder.build().ok()
								},
								_ => {
									return Err(::v_utils::__internal::eyre::eyre!("Multiple config files found: {:?}", conf_files_found));
								}
							};
							(builder.build()?, file_only)
						}
					}
				};

				// Check for unknown configuration fields
				if let Some(file_cfg) = file_config {
					Self::warn_unknown_fields(&file_cfg);
				}

				raw.try_deserialize().wrap_err(err_msg)
			}

			fn warn_unknown_fields(file_config: &::v_utils::__internal::config::Config) {
				use std::collections::{HashMap, HashSet};
				let known_fields: HashSet<&str> = [#(#field_name_strings),*].iter().copied().collect();

				if let Ok(table) = file_config.clone().try_deserialize::<HashMap<String, ::v_utils::__internal::serde_json::Value>>() {
					for field_name in table.keys() {
						if !known_fields.contains(field_name.as_str()) {
							eprintln!("warning: unknown configuration field '{field_name}' will be ignored");
						}
					}
				}
			}

			fn eval_nix_file(path: &str) -> Result<String, ::v_utils::__internal::eyre::Report> {
				use ::v_utils::__internal::eyre::WrapErr as _;
				let output = std::process::Command::new("nix")
					.arg("eval")
					.arg("--json")
					.arg("--impure")
					.arg("--expr")
					.arg(format!("import {}", path))
					.output()
					.wrap_err("Failed to execute nix command. Is nix installed?")?;

				if !output.status.success() {
					let stderr = String::from_utf8_lossy(&output.stderr);
					return Err(::v_utils::__internal::eyre::eyre!("Nix evaluation failed: {}", stderr));
				}

				Ok(String::from_utf8(output.stdout)?)
			}
		}
	};

	let flag_quotes = fields.iter().filter_map(|field| {
		let ty = &field.ty;

		// Skip Vec fields
		if let syn::Type::Path(type_path) = ty {
			if is_vec_type(type_path) {
				return None;
			}
		}

		// check if attr is `#[settings(flatten)]` or `#[settings(skip)]`
		let has_flatten_attr = field.attrs.iter().any(|attr| {
			if attr.path().is_ident("settings") {
				if let Ok(nested) = attr.parse_args::<syn::Ident>() {
					return nested == "flatten";
				}
			}
			false
		});

		let has_skip_attr = field.attrs.iter().any(|attr| {
			if attr.path().is_ident("settings") {
				if let Ok(nested) = attr.parse_args::<syn::Ident>() {
					return nested == "skip";
				}
			}
			false
		});

		// Skip fields with #[settings(skip)]
		if has_skip_attr {
			return None;
		}

		//HACK: hugely oversimplified (can only handle one level of nesting)
		let ident = &field.ident;
		Some(match has_flatten_attr {
			true => {
				use quote::ToTokens as _;
				let type_name = ty.to_token_stream().to_string();
				let nested_struct_name = format_ident!("__SettingsBadlyNested{type_name}");
				quote! {
					#[clap(flatten)]
					#ident: #nested_struct_name,
				}
			}
			false => {
				let clap_ty = clap_compatible_option_wrapped_ty(ty);
				quote! {
					#[arg(long)]
					#ident: #clap_ty,
				}
			}
		})
	});

	//HACK: code duplication. But if I produce both in single pass, it starts getting weird about types.
	let source_quotes = fields.iter().filter_map(|field| {
		let ty = &field.ty;

		// Skip Vec fields
		if let syn::Type::Path(type_path) = ty {
			if is_vec_type(type_path) {
				return None;
			}
		}

		// check if attr is `#[settings(flatten)]` or `#[settings(skip)]`
		let has_flatten_attr = field.attrs.iter().any(|attr| {
			if attr.path().is_ident("settings") {
				if let Ok(nested) = attr.parse_args::<syn::Ident>() {
					return nested == "flatten";
				}
			}
			false
		});

		let has_skip_attr = field.attrs.iter().any(|attr| {
			if attr.path().is_ident("settings") {
				if let Ok(nested) = attr.parse_args::<syn::Ident>() {
					return nested == "skip";
				}
			}
			false
		});

		// Skip fields with #[settings(skip)]
		if has_skip_attr {
			return None;
		}

		let ident = &field.ident;
		Some(match has_flatten_attr {
			true => {
				quote! {
					let _ = &self.#ident.collect_config(&mut map);
				}
			}
			false => {
				let value_kind = clap_to_config(ident.as_ref().unwrap(), ty);
				let field_name_string = format!("{}", ident.as_ref().unwrap());
				quote! {
					if let Some(#ident) = &self.#ident {
						map.insert(
							#field_name_string.to_owned(),
							v_utils::__internal::config::Value::new(Some(&"flags".to_owned()), #value_kind),
						);
					}
				}
			}
		})
	});

	let settings_args = quote_spanned! { proc_macro2::Span::call_site()=>
		//HACK: we create a struct with a fixed name here, which will error if macro is derived on more than one struct in the same scope. But good news: it's only ever meant to be derived on one struct anyways.
		#[derive(clap::Args, Clone, Debug, Default, PartialEq)] // have to derive for everything that `Cli` itself may ever want to derive.
		pub struct SettingsFlags {
			#[arg(short, long)]
			config: Option<v_utils::io::ExpandedPath>,
			#(#flag_quotes)*
		}
		impl v_utils::__internal::config::Source for SettingsFlags {
			fn clone_into_box(&self) -> Box<dyn v_utils::__internal::config::Source + Send + Sync> {
				Box::new((*self).clone())
			}

			fn collect(&self) -> Result<v_utils::__internal::config::Map<String, v_utils::__internal::config::Value>, v_utils::__internal::config::ConfigError> {
				let mut map = v_utils::__internal::config::Map::new();

				#(#source_quotes)*

				Ok(map)
			}
		}
	};
	let expanded = quote! {
		#try_build
		#settings_args
	};

	//_dbg_token_stream(expanded.clone(), "settings").into()
	TokenStream::from(expanded)
}

///NB: assumes that the child struct and the field containing it on parent are **named the same** (with adjustment for casing).
#[proc_macro_derive(SettingsBadlyNested, attributes(settings))]
pub fn derive_settings_badly_nested(input: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;
	let snake_case_name = AsSnakeCase(name.to_string()).to_string();
	let fields = if let Data::Struct(syn::DataStruct {
		fields: Fields::Named(syn::FieldsNamed { ref named, .. }),
		..
	}) = ast.data
	{
		named
	} else {
		unimplemented!()
	};

	let prefixed_flags = fields.iter().map(|field| {
		let ident = &field.ident;
		let ty = &field.ty;

		let clap_ty = clap_compatible_option_wrapped_ty(ty);
		let prefixed_field_name = format_ident!("{}_{}", snake_case_name, ident.as_ref().unwrap());
		quote! {
			#[arg(long)]
			#prefixed_field_name: #clap_ty,
		}
	});

	let config_inserts = fields.iter().map(|field| {
		let ident = &field.ident;
		let ty = &field.ty;

		let config_value_kind = clap_to_config(ident.as_ref().unwrap(), ty);
		let prefixed_field_name = format_ident!("{}_{}", snake_case_name, ident.as_ref().unwrap());
		let config_value_path = format!("{}.{}", snake_case_name, ident.as_ref().unwrap());
		let source_tag = format!("flags:{}", snake_case_name);
		quote! {
			if let Some(#ident) = &self.#prefixed_field_name {
				map.insert(
					#config_value_path.to_owned(),
					v_utils::__internal::config::Value::new(Some(&#source_tag.to_owned()), #config_value_kind),
				);
			}
		}
	});

	let produced_struct_name = format_ident!("__SettingsBadlyNested{name}");
	let expanded = quote! {
		#[derive(clap::Args, Clone, Debug, Default, PartialEq)]
		pub struct #produced_struct_name {
			#(#prefixed_flags)*
		}
		impl #produced_struct_name {
			pub fn collect_config(&self, map: &mut v_utils::__internal::config::Map<String, v_utils::__internal::config::Value>) {
				#(#config_inserts)*
			}
		}
	};

	//_dbg_token_stream(expanded.clone(), &produced_struct_name.to_string()).into()
	TokenStream::from(expanded)
}

/// Takes in field identifier and type, returns the appropriate config::ValueKind conversion
fn clap_to_config(ident: &syn::Ident, ty: &syn::Type) -> proc_macro2::TokenStream {
	// Extract the inner type from Option<T>
	let inner_type = match ty {
		syn::Type::Path(type_path) if is_option_type(type_path) => extract_option_inner_type(type_path),
		_ => ty,
	};

	match inner_type {
		// bool
		syn::Type::Path(type_path) if is_type(type_path, "bool") => {
			quote! { v_utils::__internal::config::ValueKind::Boolean(*#ident) }
		}
		// Vec/Array
		syn::Type::Path(type_path) if is_vec_type(type_path) => {
			// Create a new Vec<Value> for the array
			quote! {
				{
					let mut array = Vec::new();
					for item in #ident.iter() {
						array.push(v_utils::__internal::config::Value::new(
							None,
							v_utils::__internal::config::ValueKind::String(item.to_string())
						));
					}
					v_utils::__internal::config::ValueKind::Array(array)
				}
			}
		}
		// default to String
		_ => {
			quote! { v_utils::__internal::config::ValueKind::String(#ident.to_string()) }
		}
	}
}

// we can't do type-conversion checks at clap-parsing level, as we need to push them through config's system later.
fn clap_compatible_option_wrapped_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
	// Extract the inner type from Option<T>
	let inner_type = match ty {
		syn::Type::Path(type_path) if is_option_type(type_path) => extract_option_inner_type(type_path),
		_ => ty,
	};

	//TODO!!!!!: add numeric types (all that are in config::ValueKind)
	// Now map the inner type to clap-compatible types
	match inner_type {
		syn::Type::Path(type_path) if is_type(type_path, "bool") => {
			quote! { Option<bool> }
		}
		// If it's a Vec, check its element type
		syn::Type::Path(type_path) if is_vec_type(type_path) => {
			let vec_inner = extract_vec_inner_type(type_path);
			if let syn::Type::Path(inner_path) = vec_inner {
				if is_type(inner_path, "bool") {
					quote! { Option<Vec<bool>> }
				} else {
					// Default to Vec<String> for other element types
					quote! { Option<Vec<String>> }
				}
			} else {
				// Default to Vec<String> if we can't determine the element type
				quote! { Option<Vec<String>> }
			}
		}
		_ => quote! { Option<String> },
	}
}
//,}}}
