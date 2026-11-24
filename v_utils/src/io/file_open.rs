use std::{env, path::Path};

use eyre::{Result, WrapErr, bail, eyre};
use tokio::process::Command;

/// Position in a file (line and optional column)
#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
	pub line: u32,
	pub col: Option<u32>,
}

impl Position {
	pub fn new(line: u32, col: Option<u32>) -> Self {
		Self { line, col }
	}
}

/// Mode for opening a file
#[derive(Debug, Default)]
pub enum OpenMode {
	/// Opens file in $EDITOR, errors if file doesn't exist
	#[default]
	Normal,
	/// Opens file in $EDITOR, creates file if it doesn't exist
	Force,
	/// Opens file in nvim with readonly flag
	Read,
	/// Opens file in less pager
	Pager,
}

/// Builder for opening files with various options.
///
/// # Examples
/// ```ignore
/// use v_utils::io::file_open::Client;
///
/// // Simple open, returns whether file was modified
/// let modified = Client::default().open(&path).await?;
///
/// // Open with git sync
/// Client::default().git(true).open(&path).await?;
///
/// // Force create and open
/// Client::default().mode(OpenMode::Force).open(&path).await?;
///
/// // Open in pager with git sync
/// Client::default().git(true).mode(OpenMode::Pager).open(&path).await?;
///
/// // Open at specific line
/// Client::default().at(Position::new(42, None)).open(&path).await?;
///
/// // Open at specific line and column
/// Client::default().at(Position::new(42, Some(10))).open(&path).await?;
///
/// // Open with suggested buffer content (nvim only, user must :w to save)
/// Client::default().with_buffer("suggested content".into()).open(&path).await?;
///
/// ```
#[derive(Debug, Default)]
pub struct Client {
	git: bool,
	mode: OpenMode,
	position: Option<Position>,
	buffer: Option<String>,
}
impl Client {
	/// Enable git sync (pull before, commit+push after)
	pub fn git(mut self, enable: bool) -> Self {
		self.git = enable;
		self
	}

	/// Set the open mode
	pub fn mode(mut self, mode: OpenMode) -> Self {
		self.mode = mode;
		self
	}

	/// Set position to open file at (line and optional column)
	///
	/// Only works with known editors (nvim, helix, vscode). For unknown editors, position is ignored.
	pub fn at(mut self, position: Position) -> Self {
		self.position = Some(position);
		self
	}

	/// Pre-populate buffer with content that user must explicitly save.
	///
	/// Opens editor with the content already in buffer but marked as modified.
	/// If user quits without saving, the file remains unchanged. If user saves, content is written.
	///
	/// **nvim only** - other editors will error.
	pub fn with_buffer(mut self, contents: String) -> Self {
		self.buffer = Some(contents);
		self
	}

	/// Open the file at the given path
	///
	/// Returns `true` if file was modified, `false` otherwise.
	pub async fn open<P: AsRef<Path>>(self, path: P) -> Result<bool> {
		let path = path.as_ref();
		if self.git { self.open_with_git(path).await } else { self.open_files(&[path]).await }
	}

	/// Open the files at the given paths together: tabs in nvim/vim, however the editor takes several otherwise
	///
	/// Returns `true` if any of them was modified. A position is the first one's.
	pub async fn open_tabs<P: AsRef<Path>>(self, paths: &[P]) -> Result<bool> {
		if self.git {
			bail!("git sync opens one file");
		}
		let paths: Vec<&Path> = paths.iter().map(AsRef::as_ref).collect();
		self.open_files(&paths).await
	}

	async fn open_files(self, paths: &[&Path]) -> Result<bool> {
		let p = quoted(paths);
		let editor = Editor::detect();
		let opts = OpenOptions {
			position: self.position,
			buffer: self.buffer.as_deref(),
		};

		let mtimes = || -> Vec<_> { paths.iter().map(|path| std::fs::metadata(path).ok().and_then(|m| m.modified().ok())).collect() };
		let mtimes_before = mtimes();
		let exist = paths.iter().all(|path| path.exists());

		match self.mode {
			OpenMode::Normal => {
				if !exist {
					bail!("File does not exist");
				}
				let cmd = editor.format_open_cmd(paths, &opts)?;
				Command::new("sh").arg("-c").arg(cmd).status().await.map_err(|_| eyre!("$EDITOR env variable is not defined"))?;
			}
			OpenMode::Force => {
				let cmd = editor.format_open_cmd(paths, &opts)?;
				Command::new("sh")
					.arg("-c")
					.arg(cmd)
					.status()
					.await
					.map_err(|_| eyre!("$EDITOR env variable is not defined or permission lacking to create the file: {p}"))?;
			}
			OpenMode::Pager => {
				if !exist {
					bail!("File does not exist");
				}
				Command::new("sh").arg("-c").arg(format!("less {p}")).status().await?;
			}
			// Only works with nvim as I can't be bothered to look up "readonly" flag for all editors
			OpenMode::Read => {
				if !exist {
					bail!("File does not exist");
				}
				Command::new("sh")
					.arg("-c")
					.arg(format!("nvim -R {p}"))
					.status()
					.await
					.map_err(|_| eyre!("nvim is not found in path"))?;
			}
		}

		let modified = |(before, after)| match (before, after) {
			(None, Some(_)) => true, // file was created
			(Some(before), Some(after)) => after != before,
			_ => false,
		};
		Ok(mtimes_before.into_iter().zip(mtimes()).any(modified))
	}

	async fn open_with_git(self, path: &Path) -> Result<bool> {
		let metadata = match std::fs::metadata(path) {
			Ok(metadata) => metadata,
			Err(e) => match self.mode {
				OpenMode::Force => {
					std::fs::File::create(path).with_context(|| format!("Failed to force-create file at '{}'.\n{e}", path.display()))?;
					std::fs::metadata(path).unwrap()
				}
				_ => {
					eyre::bail!(
						"Failed to read metadata of file/directory at '{}', which means we do not have sufficient permissions or it does not exist",
						path.display()
					);
				}
			},
		};
		let sp = match metadata.is_dir() {
			true => path.display(),
			false => path.parent().unwrap().display(),
		};

		Command::new("sh").arg("-c").arg(format!("git -C \"{sp}\" pull")).status().await.with_context(|| {
			format!("Failed to pull from Git repository at '{sp}'. Ensure a repository exists at this path or any of its parent directories and no merge conflicts are present.")
		})?;

		let modified = self
			.open_files(&[path])
			.await
			.with_context(|| format!("Failed to open file at '{}'. Use `OpenMode::Force` and ensure you have necessary permissions", path.display()))?;

		Command::new("sh")
			.arg("-c")
			.arg(format!("git -C \"{sp}\" add -A && git -C \"{sp}\" commit -m \".\" && git -C \"{sp}\" push"))
			.status()
			.await
			.with_context(|| format!("Failed to commit or push to Git repository at '{sp}'. Ensure you have the necessary permissions and the repository is correctly configured."))?;

		Ok(modified)
	}
}

/// Convenience function: opens file with default settings
///
/// Returns `true` if file was modified, `false` otherwise.
pub async fn open<P: AsRef<Path>>(path: P) -> Result<bool> {
	Client::default().open(path).await
}
/// Convenience function: opens file with default settings (blocking)
///
/// Returns `true` if file was modified, `false` otherwise.
pub fn open_blocking<P: AsRef<Path>>(path: P) -> Result<bool> {
	tokio::runtime::Runtime::new().unwrap().block_on(open(path))
}
fn quoted(paths: &[&Path]) -> String {
	paths.iter().map(|p| format!("\"{}\"", p.display())).collect::<Vec<_>>().join(" ")
}

/// Options passed to editor for formatting the open command
#[derive(Debug, Default)]
struct OpenOptions<'a> {
	position: Option<Position>,
	buffer: Option<&'a str>,
}

/// Known editors with line:col support
#[derive(Clone, Copy, Debug)]
enum Editor {
	Nvim,
	Helix,
	Vscode,
	Unknown,
}

impl Editor {
	/// Detect editor from $EDITOR environment variable
	fn detect() -> Self {
		let editor = env::var("EDITOR").unwrap_or_default();
		let editor_name = Path::new(&editor).file_name().and_then(|s| s.to_str()).unwrap_or(&editor);

		match editor_name {
			"nvim" | "vim" | "vi" => Self::Nvim,
			"hx" | "helix" => Self::Helix,
			"code" | "code-insiders" => Self::Vscode,
			_ => Self::Unknown,
		}
	}

	/// Format command for opening files with given options
	fn format_open_cmd(&self, paths: &[&Path], opts: &OpenOptions) -> Result<String> {
		let p = quoted(paths);
		let tabs = if paths.len() > 1 { "-p " } else { "" };

		// Handle buffer pre-population
		if let Some(contents) = opts.buffer {
			return match self {
				Self::Nvim => {
					// Escape for lua string: backslashes and double quotes
					let escaped = contents.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
					Ok(format!(
						r#"nvim -c "lua vim.api.nvim_buf_set_lines(0, 0, -1, false, vim.split(\"{escaped}\", '\\n')); vim.bo.modified = true" {tabs}{p}"#
					))
				}
				// TODO: helix, vscode support
				_ => {
					bail!("with_buffer() only supported for nvim");
				}
			};
		}

		// Handle position
		Ok(match (self, opts.position) {
			(Self::Nvim, Some(pos)) => {
				// nvim "+call cursor(line, col) | normal zz" file - positions cursor and centers view
				match pos.col {
					Some(col) => format!("$EDITOR \"+call cursor({}, {col}) | normal zz\" {tabs}{p}", pos.line),
					None => format!("$EDITOR \"+call cursor({}, 1) | normal zz\" {tabs}{p}", pos.line),
				}
			}
			// file:line:col (both center by default)
			(Self::Helix | Self::Vscode, Some(pos)) => {
				let (first, rest) = paths.split_first().expect("opened with at least one path");
				let goto = if matches!(self, Self::Vscode) { "--goto " } else { "" };
				let col = pos.col.map(|col| format!(":{col}")).unwrap_or_default();
				format!("$EDITOR {goto}\"{}:{}{col}\" {}", first.display(), pos.line, quoted(rest))
			}
			(Self::Nvim, None) => format!("$EDITOR {tabs}{p}"),
			// Unknown editor or no position - just open the files
			(_, _) => format!("$EDITOR {p}"),
		})
	}
}
