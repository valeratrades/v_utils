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

/// Parsed field-level settings attributes
///
/// Supports:
/// - `#[settings(skip)]` - skip both flags and env
/// - `#[settings(skip(flag))]` - skip only CLI flag generation
/// - `#[settings(skip(env))]` - skip only env var binding
/// - `#[settings(skip(flag, env))]` - skip both (same as `skip`)
/// - `#[settings(flatten)]` - flatten nested struct
#[derive(Default)]
struct SettingsFieldAttrs {
	flatten: bool,
	skip_flag: bool,
	skip_env: bool,
}

impl SettingsFieldAttrs {
	fn parse(attrs: &[syn::Attribute]) -> Self {
		let mut result = Self::default();
		for attr in attrs {
			if attr.path().is_ident("settings") {
				let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
					while !input.is_empty() {
						let ident: syn::Ident = input.parse()?;
						if ident == "flatten" {
							result.flatten = true;
						} else if ident == "skip" {
							// Check if followed by parentheses with specific targets
							if input.peek(token::Paren) {
								let content;
								syn::parenthesized!(content in input);
								while !content.is_empty() {
									let target: syn::Ident = content.parse()?;
									if target == "flag" {
										result.skip_flag = true;
									} else if target == "env" {
										result.skip_env = true;
									}
									// Skip comma if present
									let _ = content.parse::<Option<Token![,]>>();
								}
							} else {
								// Plain `skip` means skip both
								result.skip_flag = true;
								result.skip_env = true;
							}
						}
						// Skip comma if present
						let _ = input.parse::<Option<Token![,]>>();
					}
					Ok(())
				});
			}
		}
		result
	}
}

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

/// Generates a custom serde Deserialize implementation for config deserialization with PrivateValue support.
///
/// This macro handles:
/// - String fields: wrapped with PrivateValue for env var support (`{ env = "VAR_NAME" }`)
/// - PathBuf fields: wrapped with ExpandedPath for tilde expansion
/// - SecretString fields: wrapped with PrivateValue and converted to SecretString
/// - Option<T> variants of the above
/// - `#[private_value]` attribute for custom types that should use PrivateValue + FromStr
/// - `#[primitives(skip)]` attribute to skip transformation for a field
/// - `#[serde(...)]` attributes are forwarded to the generated Helper struct
///
/// # Example
/// ```ignore
/// #[derive(Clone, Debug, MyConfigPrimitives)]
/// pub struct Config {
///     api_key: String,                    // Supports { env = "API_KEY" }
///     config_path: PathBuf,               // Supports ~ expansion
///     secret: SecretString,               // Supports { env = "SECRET" }
///     #[private_value]
///     port: Port,                         // Custom type via FromStr
///     #[primitives(skip)]
///     raw_value: String,                  // No transformation
/// }
/// ```
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
/// - Supports multiple config formats: TOML, JSON, YAML, and Nix
/// - Automatically searches for config files in XDG-compliant directories
/// - Generates `SettingsFlags` struct for CLI integration with clap
/// - Uses facet for deserialization with detailed error messages
/// - Nix config files are evaluated using `nix eval --json --impure` and must return a valid attribute set
/// - **Auto-extension**: When a field is missing from the config, offers to extend the config file
///   with default values (requires the struct to implement `Default + Serialize`)
///
/// # Config file resolution
/// 1. If `--config` flag is provided, uses that file (supports .nix extension)
/// 2. Otherwise, checks for `~/.config/<app_name>.nix` first
/// 3. Falls back to searching for other formats in:
///    - `~/.config/<app_name>.{toml,json,yaml}`
///    - `~/.config/<app_name>/config.{toml,json,yaml}`
///
/// # Auto-extension of Config Files
/// When the config is missing a required field, the macro will:
/// 1. Parse the error to identify the missing field
/// 2. Get the default value from `Default::default()`
/// 3. Ask the user via `confirm_blocking` if they want to extend the config
/// 4. If confirmed, add the missing field with its default value to the config file
/// 5. Retry loading the config
///
/// **Requirements for auto-extension:**
/// - The Settings struct must derive `Default` and `serde::Serialize`
/// - All nested structs must also derive `Default` and `serde::Serialize`
/// - The config file must be TOML or Nix format
///
/// **If `Default` or `Serialize` are not implemented**, the macro still compiles
/// and works normally, but the auto-extension feature is silently disabled.
/// Missing fields will just show the regular error message.
///
/// # Nesting
/// Use `#[settings(flatten)]` on fields to include nested config sections. The nested struct
/// must derive `SettingsNested`.
///
/// ```ignore
/// #[derive(Default, MyConfigPrimitives, Serialize, Settings)]
/// pub struct AppConfig {
///     pub host: String,
///     #[settings(flatten)]
///     pub database: Database,
/// }
///
/// // First level - no prefix needed, defaults to "database"
/// #[derive(Default, Deserialize, Serialize, SettingsNested)]
/// pub struct Database {
///     pub url: String,
///     #[settings(flatten)]
///     pub pool: Pool,
/// }
///
/// // Second level - must specify full prefix path
/// #[derive(Default, Deserialize, Serialize, SettingsNested)]
/// #[settings(prefix = "database_pool")]
/// pub struct Pool {
///     pub min_size: u32,
///     pub max_size: u32,
/// }
/// ```
///
/// This generates CLI flags: `--database-url`, `--database-pool-min-size`, etc.
/// Config file paths use dots: `database.url`, `database.pool.min_size`, etc.
///
/// For first-level nesting, `SettingsNested` uses the struct's snake_case name as prefix.
/// For deeper nesting, you must specify `#[settings(prefix = "parent_child")]` with the
/// full underscore-separated path.
///
/// # Example
/// ```ignore
/// #[derive(Default, MyConfigPrimitives, Serialize, Settings)]
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

	// Parse #[settings(use_env = true)] struct-level attribute
	let use_env = ast.attrs.iter().any(|attr| {
		if attr.path().is_ident("settings") {
			attr.parse_args_with(|input: syn::parse::ParseStream| {
				let ident: syn::Ident = input.parse()?;
				if ident == "use_env" {
					let _: Token![=] = input.parse()?;
					let lit: syn::LitBool = input.parse()?;
					Ok(lit.value)
				} else {
					Ok(false)
				}
			})
			.unwrap_or(false)
		} else {
			false
		}
	});

	#[cfg(feature = "xdg")]
	let xdg_conf_dir = quote_spanned! { proc_macro2::Span::call_site()=>
		let xdg_dirs = ::v_utils::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
		let xdg_conf_dir = xdg_dirs.get_config_home().unwrap().parent().unwrap().display().to_string();
	};
	#[cfg(not(feature = "xdg"))]
	let xdg_conf_dir = quote_spanned! { proc_macro2::Span::call_site()=>
		let xdg_conf_dir = ::v_utils::__internal::xdg_config_fallback();
	};

	// Generate field lists for validation (include all fields)
	// Note: #[settings(skip)], #[settings(skip(flag))], #[settings(skip(env))], and #[settings(flatten)] only affect CLI flag/env generation,
	// not config file validation - all fields are valid in config files
	let all_field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();

	let field_name_strings = all_field_names.iter().map(|name| quote! { #name });

	let try_build = quote! {
		/// Helper module for autoref specialization pattern.
		/// This allows graceful degradation when Default + Serialize are not implemented.
		mod __settings_default_provider {
			use super::*;

			pub struct Wrapper<T>(pub std::marker::PhantomData<T>);

			pub trait GetDefault<T> {
				fn get_default_for_path(&self, field_path: &str) -> Option<::v_utils::__internal::serde_json::Value>;
			}

			/// Fallback impl for reference - returns None (lower priority in method resolution)
			impl<T> GetDefault<T> for &Wrapper<T> {
				fn get_default_for_path(&self, _field_path: &str) -> Option<::v_utils::__internal::serde_json::Value> {
					None
				}
			}

			/// Impl for types that implement Default + Serialize (higher priority)
			impl<T> GetDefault<T> for Wrapper<T>
			where
				T: Default + ::v_utils::__internal::serde::Serialize,
			{
				fn get_default_for_path(&self, field_path: &str) -> Option<::v_utils::__internal::serde_json::Value> {
					let default_instance = T::default();
					let serialized = ::v_utils::__internal::serde_json::to_value(&default_instance).ok()?;

					// Navigate the path (e.g., "keybinds.movement.sneak")
					let mut current = &serialized;
					for part in field_path.split('.') {
						current = current.get(part)?;
					}
					Some(current.clone())
				}
			}

			pub trait ComputeDiff<T> {
				fn compute_diff(&self, current: &T) -> Option<String>;
			}

			/// Fallback impl for reference - returns None (lower priority in method resolution)
			impl<T> ComputeDiff<T> for &Wrapper<T> {
				fn compute_diff(&self, _current: &T) -> Option<String> {
					None
				}
			}

			/// Impl for types that implement Default + Serialize (higher priority)
			impl<T> ComputeDiff<T> for Wrapper<T>
			where
				T: Default + ::v_utils::__internal::serde::Serialize,
			{
				fn compute_diff(&self, current: &T) -> Option<String> {
					let default_instance = T::default();
					let current_json = ::v_utils::__internal::serde_json::to_value(current).ok()?;
					let default_json = ::v_utils::__internal::serde_json::to_value(&default_instance).ok()?;

					let mut diffs = Vec::new();
					collect_diffs(&current_json, &default_json, String::new(), &mut diffs);

					if diffs.is_empty() {
						None
					} else {
						Some(diffs.join("\n"))
					}
				}
			}

			fn collect_diffs(
				current: &::v_utils::__internal::serde_json::Value,
				default: &::v_utils::__internal::serde_json::Value,
				prefix: String,
				diffs: &mut Vec<String>,
			) {
				use ::v_utils::__internal::serde_json::Value;

				match (current, default) {
					(Value::Object(curr_map), Value::Object(def_map)) => {
						for (key, curr_val) in curr_map {
							let path = if prefix.is_empty() {
								key.clone()
							} else {
								format!("{}.{}", prefix, key)
							};
							if let Some(def_val) = def_map.get(key) {
								collect_diffs(curr_val, def_val, path, diffs);
							} else {
								// Field exists in current but not in default (new field)
								diffs.push(format!("{}: -> {}", path, format_value(curr_val)));
							}
						}
					}
					_ => {
						if current != default {
							diffs.push(format!("{}: {} -> {}", prefix, format_value(default), format_value(current)));
						}
					}
				}
			}

			fn format_value(value: &::v_utils::__internal::serde_json::Value) -> String {
				use ::v_utils::__internal::serde_json::Value;

				match value {
					Value::String(s) => format!("\"{}\"", s),
					Value::Null => "null".to_string(),
					Value::Bool(b) => b.to_string(),
					Value::Number(n) => n.to_string(),
					Value::Array(arr) => {
						let items: Vec<String> = arr.iter().map(format_value).collect();
						format!("[{}]", items.join(", "))
					}
					Value::Object(obj) => {
						let items: Vec<String> = obj.iter()
							.map(|(k, v)| format!("{}: {}", k, format_value(v)))
							.collect();
						format!("{{{}}}", items.join(", "))
					}
				}
			}

			pub trait GetDefaults<T> {
				fn get_defaults(&self) -> Option<::v_utils::__internal::serde_json::Value>;
			}

			/// Fallback impl for reference - returns None (lower priority in method resolution)
			impl<T> GetDefaults<T> for &Wrapper<T> {
				fn get_defaults(&self) -> Option<::v_utils::__internal::serde_json::Value> {
					None
				}
			}

			/// Impl for types that implement Default + Serialize (higher priority)
			impl<T> GetDefaults<T> for Wrapper<T>
			where
				T: Default + ::v_utils::__internal::serde::Serialize,
			{
				fn get_defaults(&self) -> Option<::v_utils::__internal::serde_json::Value> {
					let default_instance = T::default();
					::v_utils::__internal::serde_json::to_value(&default_instance).ok()
				}
			}
		}

		impl #name {
			///NB: must have `Cli` struct in the same scope, with clap derived, and `insert_clap_settings!()` macro having had been expanded inside it.
			pub fn try_build(flags: SettingsFlags) -> Result<Self, ::v_utils::__internal::eyre::Report> {
				Self::try_build_internal(flags, true)
			}

			fn try_build_internal(flags: SettingsFlags, allow_extend: bool) -> Result<Self, ::v_utils::__internal::eyre::Report> {
				let path = flags.config.as_ref().map(|p| p.0.clone());
				let app_name = env!("CARGO_PKG_NAME");

				#xdg_conf_dir

				let location_bases = [
					format!("{xdg_conf_dir}/{app_name}"),
					format!("{xdg_conf_dir}/{app_name}/config"),
				];
				let supported_exts = ["nix", "toml", "json", "yaml", "json5", "ron", "ini"];
				let locations: Vec<std::path::PathBuf> = location_bases.iter().flat_map(|base| supported_exts.iter().map(move |ext| std::path::PathBuf::from(format!("{base}.{ext}")))).collect();

				let mut builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::Environment::with_prefix(app_name).separator("__"/*default separator is '.', which I don't like being present in var names*/)).add_source(flags.clone());

				let mut err_msg = "Could not construct config from aggregated sources (conf, env, flags).".to_owned();
				#[allow(unused_imports)]
				use ::v_utils::__internal::eyre::WrapErr as _;
				let (raw, file_config, config_path): (::v_utils::__internal::config::Config, Option<::v_utils::__internal::config::Config>, Option<std::path::PathBuf>) = match path {
					Some(path) => {
						// Check if it's a .nix file
						if path.to_str().map(|s| s.ends_with(".nix")).unwrap_or(false) {
							let json_str = Self::eval_nix_file(path.to_str().unwrap())?;
							let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
							let file_only = file_builder.clone().build().ok();
							let builder = builder.add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
							(builder.build()?, file_only, Some(path))
						} else {
							let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from(path.clone()).required(true));
							let file_only = file_builder.clone().build().ok();
							let builder = builder.add_source(::v_utils::__internal::config::File::from(path.clone()).required(true));
							(builder.build()?, file_only, Some(path))
						}
					}
					None => {
						let conf_files_found: Vec<_> = locations.iter().filter(|p| p.exists()).collect();
						match conf_files_found.len() {
							0 => {
								err_msg.push_str(&format!("\nNOTE: conf file is missing. Searched in {:?}", locations));
								(builder.build()?, None, None)
							},
							1 => {
								let found_path = conf_files_found[0];
								if found_path.extension().map(|e| e == "nix").unwrap_or(false) {
									let json_str = Self::eval_nix_file(found_path.to_str().unwrap())?;
									let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
									let file_only = file_builder.clone().build().ok();
									let builder = builder.add_source(::v_utils::__internal::config::File::from_str(&json_str, ::v_utils::__internal::config::FileFormat::Json));
									(builder.build()?, file_only, Some(found_path.clone()))
								} else {
									let file_builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::File::from(found_path.as_path()).required(true));
									let file_only = file_builder.clone().build().ok();
									builder = builder.add_source(::v_utils::__internal::config::File::from(found_path.as_path()).required(true));
									(builder.build()?, file_only, Some(found_path.clone()))
								}
							},
							_ => {
								return Err(::v_utils::__internal::MultipleConfigsError {
									paths: conf_files_found.into_iter().cloned().collect(),
								}.into());
							}
						}
					}
				};

				// Check for unknown configuration fields
				if let Some(ref file_cfg) = file_config {
					Self::warn_unknown_fields(file_cfg);
				}

				// Deserialize with serde (which supports MyConfigPrimitives custom deserializer)
				match raw.try_deserialize() {
					Ok(config) => Ok(config),
					Err(e) => {
						// Check if this is a missing field error and we can extend the config
						let error_str = e.to_string();
						if allow_extend {
							if let Some(missing_field) = Self::parse_missing_field(&error_str) {
								if let Some(ref config_path) = config_path {
									// Get default values and offer to extend config
									// Uses autoref specialization: returns Some if Default+Serialize, None otherwise
									use __settings_default_provider::GetDefault as _;
									let wrapper = __settings_default_provider::Wrapper::<Self>(std::marker::PhantomData);
									if let Some(default_value) = (&wrapper).get_default_for_path(&missing_field) {
										let prompt = format!(
											"Missing configuration field \"{}\". Extend config with default value {}?",
											missing_field,
											default_value
										);
										if ::v_utils::io::confirm_blocking(&prompt) {
											if let Err(extend_err) = Self::extend_config_file(config_path, &missing_field, &default_value) {
												eprintln!("Warning: Failed to extend config: {}", extend_err);
											} else {
												eprintln!("Extended config with default for \"{}\"", missing_field);
												// Retry building - recursive call with same flags
												return Self::try_build_internal(flags, true);
											}
										}
									}
								}
							}
						}
						Err(::v_utils::__internal::eyre::eyre!("{}\n\nRoot cause: {}", err_msg, e))
					}
				}
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

			/// Parse error message to extract missing field path
			fn parse_missing_field(error_str: &str) -> Option<String> {
				// Config crate error format: `missing configuration field "field_name"`
				// or nested: `missing configuration field "parent.child.field"`
				if let Some(start) = error_str.find("missing configuration field \"") {
					let rest = &error_str[start + 29..]; // len("missing configuration field \"") = 29
					if let Some(end) = rest.find('"') {
						return Some(rest[..end].to_string());
					}
				}
				// Also try serde format: "missing field `field_name`"
				if let Some(start) = error_str.find("missing field `") {
					let rest = &error_str[start + 15..]; // len("missing field `") = 15
					if let Some(end) = rest.find('`') {
						return Some(rest[..end].to_string());
					}
				}
				None
			}


			/// Extend config file with a missing field
			fn extend_config_file(
				config_path: &std::path::Path,
				field_path: &str,
				value: &::v_utils::__internal::serde_json::Value,
			) -> Result<(), ::v_utils::__internal::eyre::Report> {
				use ::v_utils::__internal::eyre::WrapErr as _;

				let ext = config_path.extension().and_then(|e| e.to_str()).unwrap_or("");
				match ext {
					"toml" => Self::extend_toml_file(config_path, field_path, value),
					"nix" => Self::extend_nix_file(config_path, field_path, value),
					_ => Err(::v_utils::__internal::eyre::eyre!(
						"Extending config not supported for format: {}",
						ext
					)),
				}
			}

			/// Extend a TOML config file with a missing field
			fn extend_toml_file(
				config_path: &std::path::Path,
				field_path: &str,
				value: &::v_utils::__internal::serde_json::Value,
			) -> Result<(), ::v_utils::__internal::eyre::Report> {
				use ::v_utils::__internal::eyre::WrapErr as _;

				let content = std::fs::read_to_string(config_path)
					.wrap_err_with(|| format!("Failed to read config file: {}", config_path.display()))?;

				let mut doc: ::v_utils::__internal::toml::Table = content.parse()
					.wrap_err("Failed to parse TOML config")?;

				// Navigate/create the path and set the value
				let parts: Vec<&str> = field_path.split('.').collect();
				Self::set_toml_value(&mut doc, &parts, value)?;

				let new_content = ::v_utils::__internal::toml::to_string_pretty(&doc)
					.wrap_err("Failed to serialize TOML")?;

				std::fs::write(config_path, new_content)
					.wrap_err_with(|| format!("Failed to write config file: {}", config_path.display()))?;

				Ok(())
			}

			/// Helper to set a nested value in a TOML table
			fn set_toml_value(
				table: &mut ::v_utils::__internal::toml::Table,
				path: &[&str],
				value: &::v_utils::__internal::serde_json::Value,
			) -> Result<(), ::v_utils::__internal::eyre::Report> {
				if path.is_empty() {
					return Err(::v_utils::__internal::eyre::eyre!("Empty path"));
				}

				if path.len() == 1 {
					// Final key - set the value
					let toml_value = Self::json_to_toml(value)?;
					table.insert(path[0].to_string(), toml_value);
					Ok(())
				} else {
					// Need to navigate deeper
					let key = path[0];
					let nested = table.entry(key.to_string())
						.or_insert_with(|| ::v_utils::__internal::toml::Value::Table(::v_utils::__internal::toml::Table::new()));

					if let ::v_utils::__internal::toml::Value::Table(ref mut nested_table) = nested {
						Self::set_toml_value(nested_table, &path[1..], value)
					} else {
						Err(::v_utils::__internal::eyre::eyre!(
							"Expected table at '{}', found different type",
							key
						))
					}
				}
			}

			/// Convert JSON value to TOML value
			fn json_to_toml(json: &::v_utils::__internal::serde_json::Value) -> Result<::v_utils::__internal::toml::Value, ::v_utils::__internal::eyre::Report> {
				use ::v_utils::__internal::serde_json::Value as JsonValue;
				use ::v_utils::__internal::toml::Value as TomlValue;

				Ok(match json {
					JsonValue::Null => return Err(::v_utils::__internal::eyre::eyre!("TOML doesn't support null values")),
					JsonValue::Bool(b) => TomlValue::Boolean(*b),
					JsonValue::Number(n) => {
						if let Some(i) = n.as_i64() {
							TomlValue::Integer(i)
						} else if let Some(f) = n.as_f64() {
							TomlValue::Float(f)
						} else {
							return Err(::v_utils::__internal::eyre::eyre!("Unsupported number type"));
						}
					}
					JsonValue::String(s) => TomlValue::String(s.clone()),
					JsonValue::Array(arr) => {
						let toml_arr: Result<Vec<_>, _> = arr.iter().map(Self::json_to_toml).collect();
						TomlValue::Array(toml_arr?)
					}
					JsonValue::Object(obj) => {
						let mut table = ::v_utils::__internal::toml::Table::new();
						for (k, v) in obj {
							table.insert(k.clone(), Self::json_to_toml(v)?);
						}
						TomlValue::Table(table)
					}
				})
			}

			/// Extend a Nix config file with a missing field
			fn extend_nix_file(
				config_path: &std::path::Path,
				field_path: &str,
				value: &::v_utils::__internal::serde_json::Value,
			) -> Result<(), ::v_utils::__internal::eyre::Report> {
				use ::v_utils::__internal::eyre::WrapErr as _;

				let content = std::fs::read_to_string(config_path)
					.wrap_err_with(|| format!("Failed to read config file: {}", config_path.display()))?;

				// Find the position to insert the new field
				// Nix files are typically: { field1 = value1; field2 = value2; }
				// We need to find the right nesting level and insert there

				let parts: Vec<&str> = field_path.split('.').collect();
				let nix_value = Self::json_to_nix(value);

				let new_content = Self::insert_nix_field(&content, &parts, &nix_value)?;

				std::fs::write(config_path, new_content)
					.wrap_err_with(|| format!("Failed to write config file: {}", config_path.display()))?;

				Ok(())
			}

			/// Convert JSON value to Nix expression string
			fn json_to_nix(json: &::v_utils::__internal::serde_json::Value) -> String {
				use ::v_utils::__internal::serde_json::Value as JsonValue;

				match json {
					JsonValue::Null => "null".to_string(),
					JsonValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
					JsonValue::Number(n) => n.to_string(),
					JsonValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
					JsonValue::Array(arr) => {
						let items: Vec<String> = arr.iter().map(Self::json_to_nix).collect();
						format!("[ {} ]", items.join(" "))
					}
					JsonValue::Object(obj) => {
						let items: Vec<String> = obj.iter()
							.map(|(k, v)| format!("{} = {};", k, Self::json_to_nix(v)))
							.collect();
						format!("{{ {} }}", items.join(" "))
					}
				}
			}

			/// Insert a field into Nix content at the appropriate nesting level
			fn insert_nix_field(content: &str, path: &[&str], nix_value: &str) -> Result<String, ::v_utils::__internal::eyre::Report> {
				if path.is_empty() {
					return Err(::v_utils::__internal::eyre::eyre!("Empty path"));
				}

				// For simplicity, we'll handle the common case of top-level and one-level nested fields
				// More complex nesting would require a proper Nix parser

				if path.len() == 1 {
					// Top-level field: insert before the closing brace
					Self::insert_at_level(content, path[0], nix_value, 0)
				} else {
					// Nested field: find the parent block and insert there
					// First, find the parent section
					let parent_key = path[0];
					let remaining_path = &path[1..];

					// Look for the parent block: `parent_key = {`
					if let Some(parent_start) = content.find(&format!("{} = {{", parent_key))
						.or_else(|| content.find(&format!("{}={{", parent_key)))
						.or_else(|| content.find(&format!("{} ={{", parent_key)))
						.or_else(|| content.find(&format!("{}= {{", parent_key)))
					{
						// Find the opening brace after parent_key
						let brace_pos = content[parent_start..].find('{')
							.map(|p| parent_start + p)
							.ok_or_else(|| ::v_utils::__internal::eyre::eyre!("Malformed Nix: no opening brace for {}", parent_key))?;

						// Find matching closing brace
						let (block_content, close_pos) = Self::find_matching_brace(&content[brace_pos..])?;
						let close_pos = brace_pos + close_pos;

						// Recursively insert into the nested block
						let updated_block = Self::insert_nix_field(&format!("{{{}}}", block_content), remaining_path, nix_value)?;

						// Reconstruct the file
						Ok(format!(
							"{}{}{}",
							&content[..brace_pos],
							updated_block,
							&content[close_pos + 1..]
						))
					} else {
						// Parent block doesn't exist, create it with the full path
						let full_nix_value = Self::build_nested_nix(remaining_path, nix_value);
						Self::insert_at_level(content, parent_key, &full_nix_value, 0)
					}
				}
			}

			/// Build a nested Nix expression for a path
			fn build_nested_nix(path: &[&str], value: &str) -> String {
				if path.is_empty() {
					value.to_string()
				} else if path.len() == 1 {
					format!("{{ {} = {}; }}", path[0], value)
				} else {
					let inner = Self::build_nested_nix(&path[1..], value);
					format!("{{ {} = {}; }}", path[0], inner)
				}
			}

			/// Find the matching closing brace and return content between braces
			fn find_matching_brace(s: &str) -> Result<(String, usize), ::v_utils::__internal::eyre::Report> {
				let chars: Vec<char> = s.chars().collect();
				if chars.is_empty() || chars[0] != '{' {
					return Err(::v_utils::__internal::eyre::eyre!("Expected opening brace"));
				}

				let mut depth = 0;
				let mut in_string = false;
				let mut escape_next = false;

				for (i, &c) in chars.iter().enumerate() {
					if escape_next {
						escape_next = false;
						continue;
					}
					if c == '\\' && in_string {
						escape_next = true;
						continue;
					}
					if c == '"' {
						in_string = !in_string;
						continue;
					}
					if in_string {
						continue;
					}

					if c == '{' {
						depth += 1;
					} else if c == '}' {
						depth -= 1;
						if depth == 0 {
							let content: String = chars[1..i].iter().collect();
							return Ok((content, i));
						}
					}
				}

				Err(::v_utils::__internal::eyre::eyre!("No matching closing brace found"))
			}

			/// Insert a field at a specific brace nesting level
			fn insert_at_level(content: &str, key: &str, value: &str, _target_depth: usize) -> Result<String, ::v_utils::__internal::eyre::Report> {
				// Find the last closing brace at the target depth and insert before it
				let chars: Vec<char> = content.chars().collect();
				let mut depth = 0;
				let mut in_string = false;
				let mut escape_next = false;
				let mut last_close_at_depth = None;

				for (i, &c) in chars.iter().enumerate() {
					if escape_next {
						escape_next = false;
						continue;
					}
					if c == '\\' && in_string {
						escape_next = true;
						continue;
					}
					if c == '"' {
						in_string = !in_string;
						continue;
					}
					if in_string {
						continue;
					}

					if c == '{' {
						depth += 1;
					} else if c == '}' {
						if depth == 1 {
							last_close_at_depth = Some(i);
						}
						depth -= 1;
					}
				}

				if let Some(pos) = last_close_at_depth {
					// Insert the new field before the closing brace
					let (before, after) = content.split_at(pos);
					let insertion = format!("  {} = {};\n", key, value);

					// Check if we need a newline before
					let needs_newline = !before.ends_with('\n') && !before.ends_with('{');
					let prefix = if needs_newline { "\n" } else { "" };

					Ok(format!("{}{}{}{}", before, prefix, insertion, after))
				} else {
					Err(::v_utils::__internal::eyre::eyre!("Could not find insertion point in Nix file"))
				}
			}

			/// Returns a string showing fields that differ from default values.
			///
			/// Returns `None` if Default + Serialize are not implemented,
			/// or if all values match defaults.
			///
			/// Each line shows: `field.path = "value"`
			pub fn diff_from_defaults(&self) -> Option<String> {
				use __settings_default_provider::ComputeDiff as _;
				let wrapper = __settings_default_provider::Wrapper::<Self>(std::marker::PhantomData);
				(&wrapper).compute_diff(self)
			}

			/// Writes default values to the config file for fields that aren't already specified.
			///
			/// If the config file doesn't exist, creates a new one at `~/.config/<app_name>.nix`
			/// with all default values.
			///
			/// Returns `Err` if Default + Serialize are not implemented, or if file operations fail.
			pub fn write_defaults() -> Result<std::path::PathBuf, ::v_utils::__internal::eyre::Report> {
				use __settings_default_provider::GetDefaults as _;
				use ::v_utils::__internal::eyre::WrapErr as _;

				let wrapper = __settings_default_provider::Wrapper::<Self>(std::marker::PhantomData);
				let defaults = (&wrapper).get_defaults()
					.ok_or_else(|| ::v_utils::__internal::eyre::eyre!(
						"write_defaults requires Default + Serialize to be implemented on the struct"
					))?;

				let app_name = env!("CARGO_PKG_NAME");

				#xdg_conf_dir

				let location_bases = [
					format!("{xdg_conf_dir}/{app_name}"),
					format!("{xdg_conf_dir}/{app_name}/config"),
				];
				let supported_exts = ["nix", "toml", "json", "yaml", "json5", "ron", "ini"];

				// Find existing config file
				let existing_config: Option<std::path::PathBuf> = location_bases.iter()
					.flat_map(|base| supported_exts.iter().map(move |ext| std::path::PathBuf::from(format!("{base}.{ext}"))))
					.find(|p| p.exists());

				match existing_config {
					Some(config_path) => {
						// Config exists - merge missing defaults
						Self::merge_defaults_into_config(&config_path, &defaults)?;
						Ok(config_path)
					}
					None => {
						// No config exists - create new one with all defaults
						let new_config_path = std::path::PathBuf::from(format!("{xdg_conf_dir}/{app_name}.nix"));

						// Ensure parent directory exists
						if let Some(parent) = new_config_path.parent() {
							std::fs::create_dir_all(parent)
								.wrap_err_with(|| format!("Failed to create config directory: {}", parent.display()))?;
						}

						// Write defaults as Nix file
						let nix_content = Self::json_to_nix_file(&defaults);
						std::fs::write(&new_config_path, nix_content)
							.wrap_err_with(|| format!("Failed to write config file: {}", new_config_path.display()))?;

						Ok(new_config_path)
					}
				}
			}

			/// Merge missing default values into an existing config file
			fn merge_defaults_into_config(
				config_path: &std::path::Path,
				defaults: &::v_utils::__internal::serde_json::Value,
			) -> Result<(), ::v_utils::__internal::eyre::Report> {
				use ::v_utils::__internal::eyre::WrapErr as _;

				let ext = config_path.extension().and_then(|e| e.to_str()).unwrap_or("");

				// Read existing config as JSON
				let existing_json: ::v_utils::__internal::serde_json::Value = match ext {
					"nix" => {
						let json_str = Self::eval_nix_file(config_path.to_str().unwrap())?;
						::v_utils::__internal::serde_json::from_str(&json_str)
							.wrap_err("Failed to parse Nix config as JSON")?
					}
					"toml" => {
						let content = std::fs::read_to_string(config_path)
							.wrap_err_with(|| format!("Failed to read config file: {}", config_path.display()))?;
						let table: ::v_utils::__internal::toml::Table = content.parse()
							.wrap_err("Failed to parse TOML config")?;
						::v_utils::__internal::serde_json::to_value(&table)
							.wrap_err("Failed to convert TOML to JSON")?
					}
					_ => {
						// For other formats, try to read as JSON directly
						let content = std::fs::read_to_string(config_path)
							.wrap_err_with(|| format!("Failed to read config file: {}", config_path.display()))?;
						::v_utils::__internal::serde_json::from_str(&content)
							.wrap_err("Failed to parse config as JSON")?
					}
				};

				// Find fields in defaults that are missing from existing config
				let mut missing_fields = Vec::new();
				Self::find_missing_fields(defaults, &existing_json, String::new(), &mut missing_fields);

				if missing_fields.is_empty() {
					return Ok(()); // Nothing to add
				}

				// Add each missing field to the config
				for (path, value) in missing_fields {
					Self::extend_config_file(config_path, &path, &value)?;
				}

				Ok(())
			}

			/// Find fields that exist in defaults but not in existing config
			fn find_missing_fields(
				defaults: &::v_utils::__internal::serde_json::Value,
				existing: &::v_utils::__internal::serde_json::Value,
				prefix: String,
				missing: &mut Vec<(String, ::v_utils::__internal::serde_json::Value)>,
			) {
				use ::v_utils::__internal::serde_json::Value;

				if let Value::Object(def_map) = defaults {
					let existing_map = existing.as_object();

					for (key, def_val) in def_map {
						let path = if prefix.is_empty() {
							key.clone()
						} else {
							format!("{}.{}", prefix, key)
						};

						match existing_map.and_then(|m| m.get(key)) {
							Some(existing_val) => {
								// Key exists - recurse for nested objects
								if def_val.is_object() && existing_val.is_object() {
									Self::find_missing_fields(def_val, existing_val, path, missing);
								}
								// Otherwise, key exists with a value - don't override
							}
							None => {
								// Key is missing - add it
								missing.push((path, def_val.clone()));
							}
						}
					}
				}
			}

			/// Convert JSON value to a complete Nix file content
			fn json_to_nix_file(json: &::v_utils::__internal::serde_json::Value) -> String {
				Self::json_to_nix_value(json, 0)
			}

			/// Convert JSON value to Nix syntax with proper indentation
			fn json_to_nix_value(json: &::v_utils::__internal::serde_json::Value, indent: usize) -> String {
				use ::v_utils::__internal::serde_json::Value;

				let indent_str = "  ".repeat(indent);
				let inner_indent = "  ".repeat(indent + 1);

				match json {
					Value::Null => "null".to_string(),
					Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
					Value::Number(n) => n.to_string(),
					Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
					Value::Array(arr) => {
						if arr.is_empty() {
							"[]".to_string()
						} else {
							let items: Vec<String> = arr.iter()
								.map(|v| format!("{}{}", inner_indent, Self::json_to_nix_value(v, indent + 1)))
								.collect();
							format!("[\n{}\n{}]", items.join("\n"), indent_str)
						}
					}
					Value::Object(obj) => {
						if obj.is_empty() {
							"{}".to_string()
						} else {
							let items: Vec<String> = obj.iter()
								.map(|(k, v)| format!("{}{} = {};", inner_indent, k, Self::json_to_nix_value(v, indent + 1)))
								.collect();
							format!("{{\n{}\n{}}}", items.join("\n"), indent_str)
						}
					}
				}
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

		let field_attrs = SettingsFieldAttrs::parse(&field.attrs);

		// Skip fields with both skip_flag and skip_env (completely hidden from CLI)
		if field_attrs.skip_flag && field_attrs.skip_env {
			return None;
		}

		// Skip fields with skip_flag (no CLI flag generation)
		if field_attrs.skip_flag {
			return None;
		}

		let ident = &field.ident;
		Some(match field_attrs.flatten {
			true => {
				use quote::ToTokens as _;
				// Extract inner type if Option<T>
				let inner_type = if let syn::Type::Path(type_path) = ty {
					if is_option_type(type_path) { extract_option_inner_type(type_path) } else { ty }
				} else {
					ty
				};
				let type_name = inner_type.to_token_stream().to_string();
				let nested_struct_name = format_ident!("__SettingsNested{type_name}");
				quote! {
					#[clap(flatten)]
					#ident: #nested_struct_name,
				}
			}
			false => {
				let clap_ty = clap_compatible_option_wrapped_ty(ty);
				// Only add env binding if use_env is enabled AND skip_env is not set
				if use_env && !field_attrs.skip_env {
					let env_var_name = AsShoutySnakeCase(ident.as_ref().unwrap().to_string()).to_string();
					quote! {
						#[arg(long, env = #env_var_name)]
						#ident: #clap_ty,
					}
				} else {
					quote! {
						#[arg(long)]
						#ident: #clap_ty,
					}
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

		let field_attrs = SettingsFieldAttrs::parse(&field.attrs);

		// Skip fields with skip_flag (not in SettingsFlags struct, so can't collect from them)
		if field_attrs.skip_flag {
			return None;
		}

		let ident = &field.ident;
		Some(match field_attrs.flatten {
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
		#[allow(dead_code)]
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

/// Marks a struct as a nested settings section. Use with `#[settings(flatten)]` in parent.
///
/// For first-level nesting, no prefix is needed - it defaults to the struct's snake_case name.
/// For deeper nesting, specify `#[settings(prefix = "parent_child")]` with the full path.
///
/// # Example
/// ```ignore
/// // First level - no prefix needed, defaults to "database"
/// #[derive(Deserialize, SettingsNested)]
/// pub struct Database {
///     url: String,
///     #[settings(flatten)]
///     pool: Pool,
/// }
///
/// // Second level - must specify full prefix path
/// #[derive(Deserialize, SettingsNested)]
/// #[settings(prefix = "database_pool")]
/// pub struct Pool {
///     min_size: u32,
///     max_size: u32,
/// }
/// ```
///
/// This generates CLI flags like `--database-url`, `--database-pool-min-size`, etc.
#[proc_macro_derive(SettingsNested, attributes(settings))]
pub fn derive_settings_nested(input: TokenStream) -> TokenStream {
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

	// Find the optional #[settings(prefix = "...", use_env = true)] attributes
	let mut prefix = None;
	let mut use_env = false;
	for attr in &ast.attrs {
		if attr.path().is_ident("settings") {
			let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
				while !input.is_empty() {
					let ident: syn::Ident = input.parse()?;
					if ident == "prefix" {
						let _: Token![=] = input.parse()?;
						let lit: syn::LitStr = input.parse()?;
						prefix = Some(lit.value());
					} else if ident == "use_env" {
						let _: Token![=] = input.parse()?;
						let lit: syn::LitBool = input.parse()?;
						use_env = lit.value;
					}
					// Skip comma if present
					let _ = input.parse::<Option<Token![,]>>();
				}
				Ok(())
			});
		}
	}
	let prefix = prefix.unwrap_or(snake_case_name);

	// Config path uses dots (e.g., "database.pool")
	let config_prefix = prefix.replace('_', ".");

	let prefixed_flags = fields.iter().filter_map(|field| {
		let ident = &field.ident;
		let ty = &field.ty;

		let field_attrs = SettingsFieldAttrs::parse(&field.attrs);

		// Skip fields with skip_flag (no CLI flag generation)
		if field_attrs.skip_flag {
			return None;
		}

		if field_attrs.flatten {
			// For flattened nested structs, include the nested struct's flags
			use quote::ToTokens as _;
			let inner_type = if let syn::Type::Path(type_path) = ty {
				if is_option_type(type_path) { extract_option_inner_type(type_path) } else { ty }
			} else {
				ty
			};
			let type_name = inner_type.to_token_stream().to_string();
			let nested_struct_name = format_ident!("__SettingsNested{type_name}");
			Some(quote! {
				#[clap(flatten)]
				#ident: #nested_struct_name,
			})
		} else {
			let clap_ty = clap_compatible_option_wrapped_ty(ty);
			let prefixed_field_name = format_ident!("{}_{}", prefix, ident.as_ref().unwrap());
			// Only add env binding if use_env is enabled AND skip_env is not set
			if use_env && !field_attrs.skip_env {
				let env_var_name = AsShoutySnakeCase(prefixed_field_name.to_string()).to_string();
				Some(quote! {
					#[arg(long, env = #env_var_name)]
					#prefixed_field_name: #clap_ty,
				})
			} else {
				Some(quote! {
					#[arg(long)]
					#prefixed_field_name: #clap_ty,
				})
			}
		}
	});

	let config_inserts = fields.iter().filter_map(|field| {
		let ident = &field.ident;
		let ty = &field.ty;

		let field_attrs = SettingsFieldAttrs::parse(&field.attrs);

		// Skip fields with skip_flag (not in the struct, so can't collect from them)
		if field_attrs.skip_flag {
			return None;
		}

		if field_attrs.flatten {
			Some(quote! {
				let _ = &self.#ident.collect_config(map);
			})
		} else {
			let config_value_kind = clap_to_config(ident.as_ref().unwrap(), ty);
			let prefixed_field_name = format_ident!("{}_{}", prefix, ident.as_ref().unwrap());
			let config_value_path = format!("{}.{}", config_prefix, ident.as_ref().unwrap());
			let source_tag = format!("flags:{}", prefix);
			Some(quote! {
				if let Some(#ident) = &self.#prefixed_field_name {
					map.insert(
						#config_value_path.to_owned(),
						v_utils::__internal::config::Value::new(Some(&#source_tag.to_owned()), #config_value_kind),
					);
				}
			})
		}
	});

	let produced_struct_name = format_ident!("__SettingsNested{name}");
	let expanded = quote! {
		#[allow(dead_code)]
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

/// Derive macro that generates a `LiveSettings` struct for hot-reloading configuration.
/// Requires `Settings` to be derived on the same struct first.
///
/// # Generated struct
/// Creates a `LiveSettings` struct with:
/// - `new(flags: SettingsFlags, update_freq: Duration) -> Result<Self>` - constructor
/// - `config(&self) -> ConfigStruct` - returns current config, reloading if file changed
/// - `initial(&self) -> ConfigStruct` - returns initial config without reload check
///
/// # Example
/// ```ignore
/// #[derive(LiveSettings, MyConfigPrimitives, Settings)]
/// pub struct AppConfig {
///     pub host: String,
///     pub port: u16,
/// }
///
/// // Usage:
/// let live = LiveSettings::new(cli.settings, Duration::from_secs(5))?;
/// let config = live.config(); // Hot-reloads if file changed
/// ```
#[cfg(feature = "cli")]
#[proc_macro_derive(LiveSettings)]
pub fn derive_live_settings(input: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;

	#[cfg(feature = "xdg")]
	let xdg_conf_dir = quote_spanned! { proc_macro2::Span::call_site()=>
		let xdg_dirs = ::v_utils::__internal::xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));
		let xdg_conf_dir = xdg_dirs.get_config_home().unwrap().parent().unwrap().display().to_string();
	};
	#[cfg(not(feature = "xdg"))]
	let xdg_conf_dir = quote_spanned! { proc_macro2::Span::call_site()=>
		let xdg_conf_dir = ::v_utils::__internal::xdg_config_fallback();
	};

	let expanded = quote! {
		/// Thread-safe config wrapper with automatic config file hot-reload.
		/// When `config()` is called, checks if the config file has been modified
		/// since last load and reloads if necessary (with configurable throttling).
		#[derive(Clone)]
		pub struct LiveSettings {
			config_path: Option<std::path::PathBuf>,
			inner: std::sync::Arc<std::sync::RwLock<__LiveSettingsTimeCapsule>>,
			flags: SettingsFlags,
		}

		struct __LiveSettingsTimeCapsule {
			value: #name,
			loaded_at: std::time::SystemTime,
			update_freq: std::time::Duration,
		}

		impl std::fmt::Debug for LiveSettings {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.debug_struct("LiveSettings").field("config_path", &self.config_path).finish()
			}
		}

		impl LiveSettings {
			/// Create a new LiveSettings from CLI flags.
			/// `update_freq` controls how often the file modification time is checked.
			pub fn new(flags: SettingsFlags, update_freq: std::time::Duration) -> ::v_utils::__internal::eyre::Result<Self> {
				let config_path = Self::resolve_config_path(&flags)?;
				let settings = #name::try_build(flags.clone())?;

				Ok(Self {
					config_path,
					inner: std::sync::Arc::new(std::sync::RwLock::new(__LiveSettingsTimeCapsule {
						value: settings,
						loaded_at: std::time::SystemTime::now(),
						update_freq,
					})),
					flags,
				})
			}

			fn resolve_config_path(flags: &SettingsFlags) -> Result<Option<std::path::PathBuf>, ::v_utils::__internal::MultipleConfigsError> {
				if let Some(ref path) = flags.config {
					return Ok(Some(path.0.clone()));
				}

				let app_name = env!("CARGO_PKG_NAME");
				#xdg_conf_dir

				let location_bases = [
					format!("{xdg_conf_dir}/{app_name}"),
					format!("{xdg_conf_dir}/{app_name}/config"),
				];
				let supported_exts = ["nix", "toml", "json", "yaml", "json5", "ron", "ini"];

				let mut found: Vec<std::path::PathBuf> = Vec::new();
				for base in location_bases.iter() {
					for ext in supported_exts.iter() {
						let path = std::path::PathBuf::from(format!("{base}.{ext}"));
						if path.exists() {
							found.push(path);
						}
					}
				}

				match found.len() {
					0 => Ok(None),
					1 => Ok(Some(found.into_iter().next().unwrap())),
					_ => Err(::v_utils::__internal::MultipleConfigsError { paths: found }),
				}
			}

			/// Get the current settings, reloading from file if it has changed.
			pub fn config(&self) -> Result<#name, ::v_utils::__internal::MultipleConfigsError> {
				// Check for multiple configs (could have been added while running)
				Self::resolve_config_path(&self.flags)?;

				let now = std::time::SystemTime::now();

				let should_reload = {
					let capsule = self.inner.read().unwrap();
					let age = now.duration_since(capsule.loaded_at).unwrap_or_default();

					if age < capsule.update_freq {
						return Ok(capsule.value.clone());
					}

					self.config_path
						.as_ref()
						.and_then(|path| std::fs::metadata(path).ok())
						.and_then(|meta| meta.modified().ok())
						.map(|file_mtime| {
							let since_file_change = now.duration_since(file_mtime).unwrap_or_default();
							since_file_change < age
						})
						.unwrap_or(false)
				};

				if should_reload {
					if let Ok(new_settings) = #name::try_build(self.flags.clone()) {
						let mut capsule = self.inner.write().unwrap();
						capsule.value = new_settings;
						capsule.loaded_at = now;
					} else {
						let mut capsule = self.inner.write().unwrap();
						capsule.loaded_at = now;
					}
				} else {
					let mut capsule = self.inner.write().unwrap();
					capsule.loaded_at = now;
				}

				Ok(self.inner.read().unwrap().value.clone())
			}

			/// Get a direct reference to the initial settings (no reload check).
			pub fn initial(&self) -> #name {
				self.inner.read().unwrap().value.clone()
			}
		}
	};

	TokenStream::from(expanded)
}
