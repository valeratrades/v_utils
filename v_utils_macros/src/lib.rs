#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
extern crate proc_macro2;
use heck::AsShoutySnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens as _};
use syn::{
	parse::{Parse, ParseStream},
	parse_macro_input, token, Data, DeriveInput, Fields, Ident, LitInt, Token,
};

// _dbg (can't make a mod before of macro_rules not having `pub` option {{{
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
							PrivateValue::String(s) => Ok(s.clone()), //HACK: probably can avoid cloning.
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
/**
```rust
use v_utils_macros::{Settings, clap_settings};
struct Cli {
	clap_settings!()
}

//OUTDATED
let cli = Cli::parse().unwrap();
let settings = Settings::try_build(&cli).unwrap();
```
*/
//TODO!!!!!!!: \
//NB: requires `clap` to be in the scope (wouldn't make sense to bring it with the lib, as it's meant to be used in tandem and a local import will always be necessary)
#[cfg(feature = "cli")]
#[proc_macro_derive(Settings, attributes(settings))]
pub fn derive_setings(input: TokenStream) -> proc_macro::TokenStream {
	let ast = parse_macro_input!(input as syn::DeriveInput);
	let name = &ast.ident;

	let try_build = quote_spanned! {name.span()=>
		impl #name {
			///NB: must have `Cli` struct in the same scope, with clap derived, and `insert_clap_settings!()` macro having had been expanded inside it.
			#[must_use]
			pub fn try_build(flags: SettingsFlags) -> Result<Self, ::v_utils::__internal::eyre::Report> {
				let path = flags.config.as_ref().map(|p| p.0.clone());
				let app_name = env!("CARGO_PKG_NAME");
				let xdg_dirs = ::v_utils::__internal::xdg::BaseDirectories::with_prefix(app_name).unwrap(); //HACK: should use a method from `v_utils::io`, where use of `xdg` is conditional on an unrelated feature. Hardcoding `xdg` here problematic.
				let xdg_conf_dir = xdg_dirs.get_config_home().parent().unwrap().display().to_string();

				let location_bases = [
					format!("{xdg_conf_dir}/{app_name}"),
					format!("{xdg_conf_dir}/{app_name}/config"), //
				];
				let supported_exts = ["toml", "json", "yaml", "json5", "ron", "ini"];
				let locations: Vec<std::path::PathBuf> = location_bases.iter().flat_map(|base| supported_exts.iter().map(move |ext| std::path::PathBuf::from(format!("{base}.{ext}")))).collect();

				let mut builder = ::v_utils::__internal::config::Config::builder().add_source(::v_utils::__internal::config::Environment::with_prefix(app_name).separator("__"/*default separator is '.', which I don't like being present in var names*/)).add_source(flags);

				let mut err_msg = "Could not construct v_utils::__internal::config from aggregated sources (conf, env, flags, cache).".to_owned();
				use ::v_utils::__internal::eyre::WrapErr as _; //HACK: problematic as could be re-exporting
				let raw: ::v_utils::__internal::config::Config = match path {
					Some(path) => {
						let builder = builder.add_source(::v_utils::__internal::config::File::from(path.clone()).required(true));
						builder.build()?
					}
					None => {
						let mut conf_files_found = Vec::new();
						for location in locations.iter() {
							if location.exists() {
								conf_files_found.push(location);
							}
						}
						match conf_files_found.len() {
							0 => {
								err_msg.push_str(&format!("\nNOTE: conf file is missing. Searched in {:?}", locations));
							},
							1 => {
								builder = builder.add_source(::v_utils::__internal::config::File::from(conf_files_found[0].as_path()).required(true));
							},
							_ => {
								return Err(::v_utils::__internal::eyre::eyre!("Multiple config files found: {:?}", conf_files_found));
							}
						}
						builder.build()?
					}
				};
				raw.try_deserialize().wrap_err(err_msg)
			}
		}
	};

	let fields = if let syn::Data::Struct(syn::DataStruct {
		fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
		..
	}) = ast.data
	{
		named
	} else {
		unimplemented!()
	};

	let flag_quotes = fields.iter().map(|field| {
		let ty = &field.ty;

		// check if attr is `#[settings(flatten)]`
		let has_flatten_attr = field.attrs.iter().any(|attr| {
			if attr.path().is_ident("settings") {
				if let Ok(nested) = attr.parse_args::<syn::Ident>() {
					return nested == "flatten";
				}
			}
			false
		});

		//HACK: hugely oversimplified (can only handle one level of nesting)
		let ident = &field.ident;
		match has_flatten_attr {
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
				let option_wrapped = single_option_wrapped_ty(ty);
				quote! {
					#[arg(long)]
					#ident: #option_wrapped,
				}
			}
		}
	});

	//HACK: code duplication. But if I produce both in single pass, it starts getting weird about types.
	let source_quotes = fields.iter().map(|field| {
		let ty = &field.ty;

		// check if attr is `#[settings(flatten)]`
		let has_flatten_attr = field.attrs.iter().any(|attr| {
			if attr.path().is_ident("settings") {
				if let Ok(nested) = attr.parse_args::<syn::Ident>() {
					return nested == "flatten";
				}
			}
			false
		});

		let ident = &field.ident;
		match has_flatten_attr {
			true => {
				let type_name = ty.to_token_stream().to_string();
				let nested_struct_name = format_ident!("__SettingsBadlyNested{type_name}");
				quote! {
					//TODO!!!!: .
					//DO: call .collect(&mut map) on each nested struct
				}
			}
			false => {
				let value_kind = get_value_kind_for_type(ident.as_ref().unwrap(), ty);
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
		}
	});

	let settings_args = quote_spanned! { proc_macro2::Span::call_site()=>
		//HACK: we create a struct with a fixed name here, which will error if macro is derived on more than one struct in the same scope. But good news: it's only ever meant to be derived on one struct anyways.
		#[derive(Default, Debug, clap::Args, Clone, PartialEq)] // have to derive for everything that `Cli` itself may ever want to derive.
		pub struct SettingsFlags {
			#[arg(long)]
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

				if let Some(bybit_read_secret) = &self.bybit.bybit_read_secret {
					map.insert(
						"bybit.read_secret".to_owned(),
						v_utils::__internal::config::Value::new(Some(&"flags:bybit".to_owned()), v_utils::__internal::config::ValueKind::String(bybit_read_secret.to_owned())),
					);
				}

				Ok(map)
			}
		}
	};
	let expanded = quote! {
		#try_build
		#settings_args
	};

	_dbg_token_stream(expanded.clone(), "settings").into()
	//TokenStream::from(expanded)
}

#[proc_macro_derive(SettingsBadlyNested)]
pub fn derive_settings_badly_nested(input: TokenStream) -> TokenStream {
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

	let prefixed_flags = fields.iter().map(|field| {
		let ident = &field.ident;
		let ty = &field.ty;

		let option_wrapped = single_option_wrapped_ty(ty);
		let prefixed_field_name = format_ident!("{}_{}", name.to_string().to_lowercase(), ident.as_ref().unwrap());
		quote! {
			#[arg(long)]
			#prefixed_field_name: #option_wrapped,
		}
	});

	let produced_struct_name = format_ident!("__SettingsBadlyNested{name}");
	let expanded = quote! {
		#[derive(Default, Debug, clap::Args, Clone, PartialEq)]
		pub struct #produced_struct_name {
			#(#prefixed_flags)*
		}
	};

	_dbg_token_stream(expanded.clone(), &produced_struct_name.to_string()).into()
	//TokenStream::from(expanded)
}

fn single_option_wrapped_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
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

// Function to determine the ValueKind based on a field's type
fn get_value_kind_for_type(ident: &syn::Ident, ty: &syn::Type) -> proc_macro2::TokenStream {
	match ty {
		syn::Type::Path(type_path) => {
			if let syn::PathArguments::AngleBracketed(args) = &type_path.path.segments.last().unwrap().arguments {
				if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
					match inner_type {
						syn::Type::Path(inner_path) => {
							let inner_type_str = inner_path.path.segments.last().unwrap().ident.to_string();

							match inner_type_str.as_str() {
								"bool" => quote! { v_utils::__internal::config::ValueKind::Boolean(*#ident) },
								"i64" => quote! { v_utils::__internal::config::ValueKind::I64(*#ident) },
								"i128" => quote! { v_utils::__internal::config::ValueKind::I128(*#ident) },
								"u64" => quote! { v_utils::__internal::config::ValueKind::U64(*#ident) },
								"u128" => quote! { v_utils::__internal::config::ValueKind::U128(*#ident) },
								"f64" => quote! { v_utils::__internal::config::ValueKind::Float(*#ident) },
								"String" => quote! { v_utils::__internal::config::ValueKind::String(#ident.to_owned()) },
								//XXX: what on earth should happen if we need say PathBuf?
								//TODO!!!!!!!: check if this can work out at all (unlikely), and if not - only add flags of corresponding types (default to String)
								_ => quote! { v_utils::__internal::config::ValueKind::String(#ident.into()) },
							}
						}
						_ => quote! { v_utils::__internal::config::ValueKind::String(#ident.into()) },
					}
				} else {
					panic!("How did we get here?");
					quote! { v_utils::__internal::config::ValueKind::String(#ident.to_string()) }
				}
			} else {
				_dbg_tree!(type_path);
				//panic!("Surely this is impossible");
				quote! { v_utils::__internal::config::ValueKind::String(format!("{:?}", #ident))}
			}
		}
		_ => {
			panic!("Surely this is like really impossible");
			quote! { v_utils::__internal::config::ValueKind::String(#ident.to_string()) }
		}
	}
}
//,}}}
