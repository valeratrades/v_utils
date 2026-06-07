#![allow(clippy::get_first)]
#![allow(clippy::len_zero)]
#![allow(clippy::tabs_in_doc_comments)]
#![feature(stmt_expr_attributes)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![feature(default_field_values)]

#[cfg(all(feature = "wasm", feature = "full"))]
compile_error!(
	"This crate has features incompatible with each other. Do not use `--all-features`. Incompatible pairs: `wasm`+`async-io`, `wasm`+`full`, `wasm`+`xdg`. Use e.g. `--features lite` or `--features wasm` instead."
);

#[cfg(all(feature = "assert-wasm-compat", feature = "async-io"))]
compile_error!("Feature `async-io` is not compatible with wasm.");

#[cfg(all(feature = "assert-wasm-compat", feature = "full"))]
compile_error!("Feature `full` is not compatible with wasm (pulls in console-subscriber with mio).");

#[cfg(all(feature = "assert-wasm-compat", feature = "xdg"))]
compile_error!("Feature `xdg` is not compatible with wasm.");

pub mod arch;
#[cfg(feature = "bevy")]
pub mod bevy;
// of course it's included unconditionally - the crate itself is called "v_utils"
pub mod utils;

#[cfg(feature = "io")]
pub mod io;
pub mod other;
#[cfg(feature = "lite")]
pub mod prelude;
#[cfg(feature = "trades")]
pub mod trades;
#[doc(hidden)]
pub mod __internal {
	pub extern crate eyre;
	pub extern crate serde;

	#[cfg(feature = "wasm")]
	pub extern crate console_error_panic_hook;
	#[cfg(feature = "wasm")]
	pub extern crate console_log;

	#[cfg(feature = "cli")]
	pub extern crate config;
	#[cfg(feature = "cli")]
	pub extern crate facet;
	#[cfg(feature = "cli")]
	pub extern crate facet_json;
	#[cfg(feature = "cli")]
	pub extern crate facet_toml;
	#[cfg(feature = "cli")]
	pub extern crate schemars;
	#[cfg(feature = "cli")]
	pub extern crate serde_json;
	#[cfg(feature = "cli")]
	pub extern crate toml;

	#[cfg(feature = "xdg")]
	pub extern crate xdg;

	use std::path::PathBuf;

	#[cfg(all(feature = "io", not(target_arch = "wasm32")))]
	pub use crate::io::xdg::{home_dir, xdg_cache_fallback, xdg_config_fallback, xdg_data_fallback, xdg_runtime_fallback, xdg_state_fallback};

	#[cfg(feature = "cli")]
	#[derive(Debug, thiserror::Error)]
	pub enum SettingsError {
		#[error("Found multiple config files:\n{}\n\nPlease keep only one. Pick a location, merge all settings into it, then delete the rest.", .paths.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n"))]
		MultipleConfigs { paths: Vec<PathBuf> },
		/// NB: no `#[from]`/`#[source]` — these are terminal error messages, not chain links.
		/// With `#[from]`, thiserror sets `source()` to the inner type, which causes
		/// `format_eyre_chain_for_user` to print the same message twice (once as root, once as wrapper).
		#[error("{0}")]
		Parse(crate::__internal::config::ConfigError),
		#[error("{0}")]
		Other(crate::__internal::eyre::Report),
	}
	#[cfg(feature = "cli")]
	impl From<crate::__internal::config::ConfigError> for SettingsError {
		fn from(e: crate::__internal::config::ConfigError) -> Self {
			Self::Parse(e)
		}
	}
	#[cfg(feature = "cli")]
	impl From<crate::__internal::eyre::Report> for SettingsError {
		fn from(e: crate::__internal::eyre::Report) -> Self {
			Self::Other(e)
		}
	}

	#[cfg(feature = "cli")]
	impl crate::utils::SysexitCode for SettingsError {
		fn sysexit(&self) -> crate::utils::Sysexit {
			crate::utils::Sysexit::Config
		}
	}

	/// Translate a `schemars` JSON Schema into a NixOS-style options module.
	///
	/// Lives in `__internal` because its only caller is the `write_module()` method generated
	/// by `#[derive(Settings)]` — it is not part of the human-facing API. The generated module
	/// is *options-only*: it declares the exact field names and types (and descriptions, when
	/// the schema carries them) so a config that `import`s it gets eval-time type checking and
	/// editor awareness via `nixd`/`nil`. It deliberately bakes in NO value-defaults — Rust's
	/// `Default` owns those; the user's config sets the values. The single exception is
	/// `Option<T>` fields, which get `default = null;` so they may legitimately be omitted.
	///
	/// The emitted file is a function `{ lib, ... }: { options = { … }; }`, ready to be a module
	/// in a `lib.evalModules { modules = [ ./this.nix ./user-config.nix ]; }` evaluation.
	#[cfg(feature = "cli")]
	pub fn schema_to_nix_module(schema: &crate::__internal::serde_json::Value) -> Result<String, crate::__internal::eyre::Report> {
		use crate::__internal::{
			eyre::{OptionExt as _, eyre},
			serde_json::Value,
		};

		let defs = schema.get("$defs").and_then(Value::as_object);

		/// Resolve a possibly-`$ref` schema node to its concrete definition.
		fn resolve<'a>(node: &'a Value, defs: Option<&'a serde_json::Map<String, Value>>) -> Result<&'a Value, crate::__internal::eyre::Report> {
			if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
				let name = reference.strip_prefix("#/$defs/").ok_or_else(|| eyre!("unsupported $ref form: {reference}"))?;
				let defs = defs.ok_or_else(|| eyre!("schema has a $ref but no $defs"))?;
				return defs.get(name).ok_or_else(|| eyre!("dangling $ref: {reference}"));
			}
			Ok(node)
		}

		/// Map a single (resolved-on-demand) schema node to a `lib.types.<…>` expression.
		/// `depth` is the indentation level of the line this expression is emitted on, so that a
		/// nested `submodule { options = …; }` indents its contents relative to that line.
		fn nix_type(node: &Value, defs: Option<&serde_json::Map<String, Value>>, depth: usize) -> Result<String, crate::__internal::eyre::Report> {
			let node = resolve(node, defs)?;

			// `Option<T>` is encoded as `"type": ["T", "null"]`. Peel the null and wrap in nullOr.
			if let Some(arr) = node.get("type").and_then(Value::as_array) {
				let non_null: Vec<&Value> = arr.iter().filter(|t| t.as_str() != Some("null")).collect();
				let inner = non_null.first().ok_or_eyre("type array with only null")?;
				// Reconstruct a single-typed node so the scalar branch below handles it.
				let mut single = node.clone();
				single["type"] = (*inner).clone();
				return Ok(format!("lib.types.nullOr {}", nix_type(&single, defs, depth)?));
			}

			// Unit-variant enums: `"type": "string", "enum": [...]`.
			if let Some(variants) = node.get("enum").and_then(Value::as_array) {
				let items = variants
					.iter()
					.map(|v| v.as_str().map(|s| format!("\"{s}\"")).ok_or_eyre("non-string enum variant"))
					.collect::<Result<Vec<_>, _>>()?;
				return Ok(format!("lib.types.enum [ {} ]", items.join(" ")));
			}

			match node.get("type").and_then(Value::as_str) {
				Some("string") => Ok("lib.types.str".to_string()),
				Some("boolean") => Ok("lib.types.bool".to_string()),
				Some("integer") => Ok("lib.types.int".to_string()),
				Some("number") => Ok("lib.types.float".to_string()),
				Some("array") => {
					let items = node.get("items").ok_or_eyre("array schema without `items`")?;
					Ok(format!("lib.types.listOf {}", nix_type(items, defs, depth)?))
				}
				Some("object") => {
					// Free-form map (`HashMap<String, V>`) vs a struct with named properties.
					if let Some(additional) = node.get("additionalProperties") {
						if additional.is_object() {
							return Ok(format!("lib.types.attrsOf {}", nix_type(additional, defs, depth)?));
						}
					}
					Ok(format!("lib.types.submodule {{ options = {}; }}", options_block(node, defs, depth)?))
				}
				other => Err(eyre!("unsupported schema type: {other:?}")),
			}
		}

		/// Build the `{ <field> = lib.mkOption {...}; ... }` block for an object node.
		fn options_block(obj: &Value, defs: Option<&serde_json::Map<String, Value>>, depth: usize) -> Result<String, crate::__internal::eyre::Report> {
			let properties = obj.get("properties").and_then(Value::as_object);
			let Some(properties) = properties else {
				// An object with no declared properties is a degenerate (empty) submodule.
				return Ok("{ }".to_string());
			};
			let required: std::collections::HashSet<&str> = obj
				.get("required")
				.and_then(Value::as_array)
				.map(|a| a.iter().filter_map(Value::as_str).collect())
				.unwrap_or_default();

			let indent = "  ".repeat(depth);
			let inner_indent = "  ".repeat(depth + 1);
			let mut lines = Vec::new();
			for (field, node) in properties {
				let ty = nix_type(node, defs, depth + 1)?;
				let description = resolve(node, defs)?.get("description").and_then(Value::as_str);
				// Optional fields (absent from `required`, i.e. `Option<T>`) get `default = null;`
				// so the user may omit them; required fields stay mandatory (no default).
				let is_optional = !required.contains(field.as_str());

				let mut parts = vec![format!("type = {ty};")];
				if let Some(desc) = description {
					parts.push(format!("description = \"{}\";", desc.replace('\\', "\\\\").replace('"', "\\\"")));
				}
				if is_optional {
					parts.push("default = null;".to_string());
				}
				lines.push(format!("{inner_indent}{field} = lib.mkOption {{ {} }};", parts.join(" ")));
			}
			Ok(format!("{{\n{}\n{indent}}}", lines.join("\n")))
		}

		if schema.get("type").and_then(Value::as_str) != Some("object") {
			bail!("top-level settings schema must be an object, got {:?}", schema.get("type"));
		}
		let options = options_block(schema, defs, 1)?;
		Ok(format!("{{ lib, ... }}:\n{{\n  options = {options};\n}}\n"))
	}
}
#[cfg(feature = "distributions")]
pub mod distributions;
#[cfg(test)]
pub(crate) mod internal_utils;

//Q: I like the idea of having a prelude, but atm it just leads to possibility of mismatching def paths, client imports v_utils and something else relying on a different version of v_utils

pub use other::*;

#[cfg(feature = "macros")]
pub extern crate v_utils_macros as macros;
