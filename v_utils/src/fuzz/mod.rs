//! Deterministic-fuzz kernel: matklad's FRNG + TigerBeetle's VOPR loop, minus everything
//! domain-specific. A consumer supplies a table of [`Target`]s and a corpus path, and gets the two
//! tests back:
//!
//! ```ignore
//! const SUITE: Suite = Suite {
//!     targets: TARGETS,
//!     corpus: concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fuzz/CORPUS.txt"),
//! };
//!
//! #[test] fn fuzz() { SUITE.fuzz() }
//! #[test] fn regressions() { SUITE.regressions() }
//! ```
//!
//! [`Suite::fuzz`] drives random seeds against each target, auto-minimizes the first failure and
//! records its minimal `(target, seed, size)`; [`Suite::regressions`] replays every recorded case
//! that is still at its target's current fingerprint. Env-var replay: `FUZZ_SEED` / `FUZZ_SIZE` /
//! `FUZZ_TARGET` / `FUZZ_RUNS`.
//!
//! `corpus` and [`FRNG_SRC`] are values rather than `include_str!`/`env!` here on purpose: both
//! macros resolve against the crate being compiled, so a path or a source text a consumer needs
//! must be handed over as data.

mod frng;
mod minimize;

use std::{cell::RefCell, fs::OpenOptions, io::Write, path::Path};

pub use frng::*;
pub use minimize::*;

/// The generator source every consumer's fingerprint has to include, since the buffer a
/// `(seed, size)` names is this file's arithmetic.
pub const FRNG_SRC: &str = include_str!("frng.rs");

/// Default fuzz budget. `FUZZ_RUNS` / `FUZZ_SIZE` override; bump locally for deeper runs.
const DEFAULT_RUNS: u64 = 512;
const DEFAULT_SIZE: usize = 256;

/// FNV-1a over the sources that decide what a `(seed, size)` means. Derived rather than hand-bumped:
/// a version someone has to remember to raise is a version that silently goes stale, and the failure
/// mode there is a corpus quietly passing without testing anything. Over-sensitive on purpose (a
/// comment edit invalidates too): the cost of a false invalidation is a few free runs, the cost of a
/// false match is a lie.
pub const fn fnv(sources: &[&str]) -> u32 {
	let mut h: u32 = 0x811c_9dc5;
	let mut i = 0;
	while i < sources.len() {
		let bytes = sources[i].as_bytes();
		let mut j = 0;
		while j < bytes.len() {
			h = (h ^ bytes[j] as u32).wrapping_mul(0x0100_0193);
			j += 1;
		}
		i += 1;
	}
	h
}

/// One property, generator and oracle together. `run` reads its whole trace off a [`Frng`] built
/// from `(seed, size)`, so that pair names it exactly; `version` fingerprints the sources that
/// decide what the pair means, and is per target — one fingerprint over a whole binary would let an
/// edit to one generator retire another's corpus.
pub struct Target {
	pub name: &'static str,
	pub version: u32,
	pub run: fn(u64, usize, bool) -> Result<(), String>,
}

pub struct Suite<'a> {
	pub targets: &'a [Target],
	/// Absolute path to the corpus file; `concat!(env!("CARGO_MANIFEST_DIR"), …)` at the call site,
	/// so it resolves regardless of the test runner's working directory.
	pub corpus: &'a str,
}

impl Suite<'_> {
	/// Scan → minimize → replay → record → panic. Installs [`quiet_hook`], so anything this binary
	/// runs afterwards reports its panics through [`replay`] rather than to stderr.
	pub fn fuzz(&self) {
		quiet_hook();
		let size = env_usize("FUZZ_SIZE", DEFAULT_SIZE);
		let only = std::env::var("FUZZ_TARGET").ok();
		let picked = || self.targets.iter().filter(|t| only.as_ref().is_none_or(|n| *n == t.name));
		assert!(
			picked().next().is_some(),
			"FUZZ_TARGET names no target; the set is {:?}",
			self.targets.iter().map(|t| t.name).collect::<Vec<_>>()
		);

		if let Ok(s) = std::env::var("FUZZ_SEED") {
			let seed = s.parse().expect("FUZZ_SEED must be a u64");
			for t in picked() {
				replay(t, seed, size);
			}
			return;
		}

		let runs: u64 = std::env::var("FUZZ_RUNS").ok().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_RUNS);
		for t in picked() {
			for seed in 0..runs {
				if !fails(t, seed, size) {
					continue;
				}
				let (ms, msz) = minimize(&|s, z| fails(t, s, z), seed, size);
				eprintln!("\n=== FUZZ FAILURE ({}, seed={seed}, size={size}) ===", t.name);
				eprintln!("minimal repro: (target={}, seed={ms}, size={msz})", t.name);
				let reason = replay(t, ms, msz).unwrap_or_else(|| "(no failure on replay)".to_string());
				self.record(t.name, ms, msz, t.version, &reason);
				eprintln!("recorded to {} — fix the bug, then commit the new line.", self.corpus);
				panic!("fuzz found a failure; minimal repro (target={}, seed={ms}, size={msz}): {reason}", t.name);
			}
			eprintln!("fuzz: {} clean over {runs} seeds at size {size}", t.name);
		}
	}

	/// Replay every recorded case whose target still exists at the version it was recorded under.
	pub fn regressions(&self) {
		quiet_hook();
		let all = self.load();
		let mut live = 0usize;
		for e in &all {
			let Some(t) = self.targets.iter().find(|t| t.name == e.target) else {
				continue; // a target that was removed: the line stays as documentation
			};
			if e.version != t.version {
				continue;
			}
			live += 1;
			assert!(!fails(t, e.seed, e.size), "recorded regression ({}, seed={}, size={}) fails again", e.target, e.seed, e.size);
		}
		// Loud, because a corpus that has silently emptied itself still reports green.
		eprintln!(
			"corpus: replayed {live} of {} recorded cases; the rest were recorded under an older generator and are history, not tests",
			all.len()
		);
		for t in self.targets {
			eprintln!("  {} at generator {:08x}", t.name, t.version);
		}
	}

	/// Parse the corpus: one `target seed size version` per line; `#` starts a comment; blanks
	/// ignored. A line that does not parse is corruption and panics — skipping it would turn a loud
	/// error into a silently shrinking corpus, which is the exact failure this file exists to avoid.
	fn load(&self) -> Vec<Entry> {
		let text = std::fs::read_to_string(self.corpus).unwrap_or_default();
		let mut out = Vec::new();
		for line in text.lines() {
			let data = line.split('#').next().unwrap_or("").trim();
			if data.is_empty() {
				continue;
			}
			let mut it = data.split_whitespace();
			let target = it.next().expect("corpus line has a target").to_owned();
			let seed = it.next().expect("corpus line has a seed").parse().expect("corpus seed is a u64");
			let size = it.next().expect("corpus line has a size").parse().expect("corpus size is a usize");
			let version = it.next().expect("corpus line has a version").parse().expect("corpus version is a u32");
			out.push(Entry { target, seed, size, version });
		}
		out
	}

	/// Append a newly-found minimal repro unless it's already recorded at this generator version.
	/// `reason` is written as a trailing comment so a reviewer of the diff sees what broke.
	fn record(&self, target: &str, seed: u64, size: usize, version: u32, reason: &str) {
		if self.load().iter().any(|e| e.target == target && e.seed == seed && e.size == size && e.version == version) {
			return;
		}
		let line = format!("{target} {seed} {size} {version}  # {}\n", reason.replace('\n', " "));
		// One short append on the rare failure path; the file is only ever appended to.
		let mut f = OpenOptions::new().create(true).append(true).open(Path::new(self.corpus)).expect("open corpus for append");
		f.write_all(line.as_bytes()).expect("append to corpus");
	}
}

struct Entry {
	target: String,
	seed: u64,
	size: usize,
	version: u32,
}

thread_local! {
	static LAST_PANIC: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Swallow the default panic backtrace (we catch and report ourselves), but stash `info` — the
/// `catch_unwind` payload downcasts to the message alone, so the `file:line` a recorded `PANIC:`
/// line carries is only recoverable from here.
///
/// This mutes the *whole test binary*, including any test not driven through a [`Suite`].
fn quiet_hook() {
	static ONCE: std::sync::Once = std::sync::Once::new();
	ONCE.call_once(|| {
		std::panic::set_hook(Box::new(|info| {
			LAST_PANIC.with(|c| *c.borrow_mut() = info.to_string());
		}));
	});
}

fn env_usize(key: &str, default: usize) -> usize {
	std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

/// Run one case under `catch_unwind`, so a production `expect`/`unreachable` blowup counts as a
/// failure exactly as a violated oracle does. Free fn over a bare fn pointer: `&self` would drag
/// `Suite` across the unwind boundary for nothing.
fn fails(t: &Target, seed: u64, size: usize) -> bool {
	let run = t.run;
	std::panic::catch_unwind(move || run(seed, size, false)).map(|r| r.is_err()).unwrap_or(true)
}

/// Verbose re-run of one case: the target prints its own trace, and this reports the failure (or the
/// panic). Returns the failure reason, or `None` if it didn't reproduce.
fn replay(t: &Target, seed: u64, size: usize) -> Option<String> {
	eprintln!("--- replay (target={}, seed={seed}, size={size}) ---", t.name);
	let run = t.run;
	match std::panic::catch_unwind(move || run(seed, size, true)) {
		Ok(Ok(())) => {
			eprintln!("(no failure on replay)");
			None
		}
		Ok(Err(what)) => {
			eprintln!("FAILURE: {what}");
			Some(what)
		}
		Err(_) => {
			let p = LAST_PANIC.with(|c| c.borrow().clone());
			eprintln!("PANIC: {p}");
			Some(format!("PANIC: {p}"))
		}
	}
}
