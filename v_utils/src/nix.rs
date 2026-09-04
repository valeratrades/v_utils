//! Reading `.nix` config files without paying `nix` binary startup (~36ms) on every config load.
//!
//! Every observed config is a plain attrset of literals, which we convert in-process. Anything that
//! uses the Nix *language* is handed to `nix eval`, which stays authoritative for both result and
//! error. The branch is picked by syntax alone, before any evaluation — `pure_data_json` returns
//! `None` for "not literal data", never for "evaluation failed", so there is no error to swallow.

use std::path::Path;

use eyre::{Report, WrapErr as _, bail};
use rnix::ast::{self, Expr, HasEntry as _, InterpolPart};
use serde_json::{Map, Number, Value};

/// JSON text of the attrset in `path`.
pub fn eval_nix_file(path: &Path) -> Result<String, Report> {
	let src = std::fs::read_to_string(path).wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;
	// An empty `.nix` file is invalid Nix and yields a cryptic `unexpected end of file`; surface the real cause.
	if src.trim().is_empty() {
		bail!("Config file `{}` is empty. Delete it to write a fresh default config, or fill in valid Nix.", path.display());
	}

	match pure_data_json(&src) {
		Some(value) => Ok(value.to_string()),
		None => nix_eval(path),
	}
}

fn nix_eval(path: &Path) -> Result<String, Report> {
	// `--file` over `--expr "import {path}"`: the path becomes an argv element, so spaces and
	// relative paths stop being reinterpreted as Nix syntax.
	let output = std::process::Command::new("nix")
		.args(["eval", "--json", "--impure", "--file"])
		.arg(path)
		.output()
		.wrap_err("Failed to execute nix command. Is nix installed?")?;

	if !output.status.success() {
		bail!("Nix evaluation failed: {}", String::from_utf8_lossy(&output.stderr));
	}
	Ok(String::from_utf8(output.stdout)?)
}

/// `None` == "not literal data, ask nix". Every unhandled node must land in a `_` arm — never `if
/// let` past an unknown one, or an unsupported construct silently becomes a wrong value.
fn pure_data_json(src: &str) -> Option<Value> {
	let parse = rnix::Root::parse(src);
	// invalid Nix: delegate, so the user gets nix's diagnostic rather than rnix's
	if !parse.errors().is_empty() {
		return None;
	}
	match parse.tree().expr()? {
		Expr::AttrSet(set) => attrset_to_json(&set),
		_ => None,
	}
}

fn attrset_to_json(set: &ast::AttrSet) -> Option<Value> {
	if set.rec_token().is_some() {
		return None;
	}
	let mut out = Map::new();
	for entry in set.entries() {
		let ast::Entry::AttrpathValue(av) = entry else { return None }; // `inherit`
		let keys = av.attrpath()?.attrs().map(|a| attr_name(&a)).collect::<Option<Vec<_>>>()?;
		insert_path(&mut out, &keys, expr_to_json(&av.value()?)?)?;
	}
	Some(Value::Object(out))
}

/// Nix merges attrsets across duplicate attrpaths and errors on every other collision; `None` hands
/// that error to nix rather than guessing at last-wins.
fn insert_path(map: &mut Map<String, Value>, keys: &[String], value: Value) -> Option<()> {
	let (head, rest) = keys.split_first()?;
	if rest.is_empty() {
		return match (map.get_mut(head), value) {
			(None, value) => {
				map.insert(head.clone(), value);
				Some(())
			}
			(Some(Value::Object(existing)), Value::Object(new)) => new.into_iter().try_for_each(|(k, v)| insert_path(existing, &[k], v)),
			(Some(_), _) => None,
		};
	}
	match map.entry(head.clone()).or_insert_with(|| Value::Object(Map::new())) {
		Value::Object(child) => insert_path(child, rest, value),
		_ => None,
	}
}

fn attr_name(attr: &ast::Attr) -> Option<String> {
	match attr {
		ast::Attr::Ident(ident) => Some(ident.ident_token()?.text().to_owned()),
		ast::Attr::Str(s) => str_literal(s),
		ast::Attr::Dynamic(_) => None,
	}
}

fn str_literal(s: &ast::Str) -> Option<String> {
	// `''` strings add indent-stripping whose failure mode is a silently wrong string. Delegating
	// costs two lines; to accept them, drop this and extend the differential corpus.
	if s.to_string().starts_with("''") {
		return None;
	}
	match s.normalized_parts().as_slice() {
		[] => Some(String::new()),
		[InterpolPart::Literal(text)] => Some(text.clone()),
		_ => None,
	}
}

fn expr_to_json(expr: &Expr) -> Option<Value> {
	match expr {
		Expr::AttrSet(set) => attrset_to_json(set),
		Expr::List(list) => list.items().map(|i| expr_to_json(&i)).collect::<Option<Vec<_>>>().map(Value::Array),
		Expr::Str(s) => str_literal(s).map(Value::String),
		Expr::Literal(lit) => literal_to_json(lit, false),
		Expr::UnaryOp(op) => match (op.operator()?, op.expr()?) {
			(ast::UnaryOpKind::Negate, Expr::Literal(lit)) => literal_to_json(&lit, true),
			_ => None,
		},
		// Nix has no `true`/`false`/`null` keywords — they are ordinary, shadowable variables
		// (`let true = 5; in { x = true; }` evaluates to `{"x":5}`). This mapping is sound only
		// because every binding form (`let`, `with`, `rec`, `inherit`, lambda) is rejected above.
		Expr::Ident(ident) => match ident.ident_token()?.text() {
			"true" => Some(Value::Bool(true)),
			"false" => Some(Value::Bool(false)),
			"null" => Some(Value::Null),
			_ => None,
		},
		_ => None,
	}
}

fn literal_to_json(lit: &ast::Literal, negate: bool) -> Option<Value> {
	match lit.kind() {
		// nix rejects `9223372036854775808` itself, so negation can never overflow here
		ast::LiteralKind::Integer(i) => {
			let v = i.value().ok()?;
			Some(Value::Number(if negate { -v } else { v }.into()))
		}
		ast::LiteralKind::Float(f) => {
			let v = f.value().ok()?;
			Number::from_f64(if negate { -v } else { v }).map(Value::Number)
		}
		ast::LiteralKind::Uri(_) => None,
	}
}

/// Differential against real `nix` — it is the oracle, so no expected values are hand-written here.
#[cfg(test)]
mod tests {
	use std::{io::Write as _, path::Path, process::Command};

	use super::*;

	/// Must be handled in-process, byte-for-byte as nix would.
	const PURE: &[&str] = &[
		"{}",
		"{ a = 1; b = 1.0; c = -3; d = -1.5; e = 1.0e10; f = 0.1; }",
		"{ a = 9223372036854775807; b = -9223372036854775807; }",
		"{ t = true; f = false; n = null; }",
		r#"{ s = ""; tab = "a\tb"; q = "a\"b"; bs = "a\\b"; dollar = "a\${x}b"; nl = "a\nb"; }"#,
		r#"{ u = "ы 日本 🦀"; }"#,
		"{ l = []; m = [ 1 \"a\" [ null { k = 2; } ] ]; }",
		// shape of ~/.config/tedi.nix — `{ env = ...; }` is a sentinel the Rust side reads, not substitution
		r#"{
			manual_stats = { date_format = "%Y-%m-%d"; };
			github_token = { env = "GITHUB_KEY"; };
			milestones = { url = "https://github.com/valeratrades/todos"; };
			timer = { hard_stop_coeff = 1.5; };
		}"#,
		// v_utils_macros/tests/example_config.nix
		"{\n  # Simple values\n  name = \"test_app\";\n  value = 42;\n\n  # Boolean\n  debug = true;\n}",
		r#"{ "with-dash" = 1; "a.b" = 2; "" = 3; }"#,
		"{ a.b.c = 1; a.b.d = 2; a.e = 3; }",
		"{ a = { b = 1; }; a.c = 2; }",
		"{ a = { b = 1; }; a = { c = 2; }; }",
		"{ a = { b = { c = { d = { e = 1; }; }; }; }; }",
		"# lead\n{ /* mid */ a = /* inline */ 1; # trail\n}\n",
	];

	/// Valid nix outside the subset: classified out, still evaluated identically via `nix eval`.
	const DELEGATED: &[&str] = &[
		"{ lib, ... }: { a = 1; }",
		"let x = 1; in { a = x; }",
		"rec { a = 1; b = a; }",
		"let a = 1; in { inherit a; }",
		"{ a = { b = 1; } // { c = 2; }; }",
		"{ a = 1 + 2; }",
		r#"{ a = "pre${builtins.getEnv "PATH"}post"; }"#,
		"{ a = ''\n  hi\n  there\n''; }",
		"{ a = if true then 1 else 2; }",
		"{ a = (1); }",
		"{ ${\"dyn\"} = 1; }",
		"with { x = 1; }; { a = x; }",
	];

	/// Rejected by nix — the fast path must never accept what nix refuses.
	const INVALID: &[&str] = &[
		"{ a = 1; a = 2; }",
		"{ a.b = 1; a.b = 2; }",
		"{ a = 1; a.b = 2; }",
		"{ a = { b = 1; }; a = { b = 2; }; }",
		"{ a = 1;",
		"{ a = foo; }",
		"{ a = ./relative_that_does_not_exist.nix; }",
		"import ./other.nix",
	];

	fn write(src: &str) -> tempfile::NamedTempFile {
		let mut f = tempfile::Builder::new().suffix(".nix").tempfile().unwrap();
		f.write_all(src.as_bytes()).unwrap();
		f.flush().unwrap();
		f
	}

	fn oracle(path: &Path) -> Option<Value> {
		let out = Command::new("nix").args(["eval", "--json", "--impure", "--file"]).arg(path).output().unwrap();
		out.status.success().then(|| serde_json::from_slice(&out.stdout).unwrap())
	}

	#[test]
	fn matches_nix() {
		// skipping when nix is absent would let this pass while proving nothing; the repo has a flake
		assert!(Command::new("nix").arg("--version").output().is_ok_and(|o| o.status.success()), "`nix` is required by this test");

		for src in PURE {
			let f = write(src);
			assert_eq!(pure_data_json(src), oracle(f.path()), "{src}");
		}
		for src in DELEGATED.iter().chain(INVALID) {
			let f = write(src);
			assert_eq!(pure_data_json(src), None, "{src}");
			let ours = eval_nix_file(f.path()).ok().map(|s| serde_json::from_str(&s).unwrap());
			assert_eq!(ours, oracle(f.path()), "{src}");
		}
	}

	#[test]
	fn empty_file_names_itself() {
		let f = write("  \n");
		let err = eval_nix_file(f.path()).unwrap_err().to_string();
		assert!(err.contains("is empty"), "{err}");
	}
}
