use std::{env, path::Path};

use eyre::{Result, WrapErr, bail, eyre};
use tokio::{process::Command, sync::oneshot};

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

	/// Format command arguments for opening file at position
	fn format_open_cmd(&self, path: &Path, position: Option<Position>) -> String {
		let p = path.display();
		match (self, position) {
			(Self::Nvim, Some(pos)) => {
				// nvim +line file or nvim "+call cursor(line, col)" file
				match pos.col {
					Some(col) => format!("$EDITOR \"+call cursor({}, {col})\" \"{p}\"", pos.line),
					None => format!("$EDITOR +{} \"{p}\"", pos.line),
				}
			}
			(Self::Helix, Some(pos)) => {
				// helix file:line:col
				match pos.col {
					Some(col) => format!("$EDITOR \"{p}:{}:{col}\"", pos.line),
					None => format!("$EDITOR \"{p}:{}\"", pos.line),
				}
			}
			(Self::Vscode, Some(pos)) => {
				// code --goto file:line:col
				match pos.col {
					Some(col) => format!("$EDITOR --goto \"{p}:{}:{col}\"", pos.line),
					None => format!("$EDITOR --goto \"{p}:{}\"", pos.line),
				}
			}
			// Unknown editor or no position - just open the file
			(_, _) => format!("$EDITOR \"{p}\""),
		}
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
/// // Simple open
/// Client::default().open(&path).await?;
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
/// ```
#[derive(Debug, Default)]
pub struct Client {
	git: bool,
	mode: OpenMode,
	position: Option<Position>,
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

	/// Open the file at the given path
	pub async fn open<P: AsRef<Path>>(self, path: P) -> Result<()> {
		let path = path.as_ref();
		if self.git { self.open_with_git(path).await } else { self.open_file(path).await }
	}

	async fn open_file(self, path: &Path) -> Result<()> {
		let p = path.display();
		let editor = Editor::detect();
		match self.mode {
			OpenMode::Normal => {
				if !path.exists() {
					bail!("File does not exist");
				}
				let cmd = editor.format_open_cmd(path, self.position);
				Command::new("sh").arg("-c").arg(cmd).status().await.map_err(|_| eyre!("$EDITOR env variable is not defined"))?;
			}
			OpenMode::Force => {
				let cmd = editor.format_open_cmd(path, self.position);
				Command::new("sh")
					.arg("-c")
					.arg(cmd)
					.status()
					.await
					.map_err(|_| eyre!("$EDITOR env variable is not defined or permission lacking to create the file: {p}"))?;
			}
			OpenMode::Pager => {
				if !path.exists() {
					bail!("File does not exist");
				}
				Command::new("sh").arg("-c").arg(format!("less {p}")).status().await?;
			}
			// Only works with nvim as I can't be bothered to look up "readonly" flag for all editors
			OpenMode::Read => {
				if !path.exists() {
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

		Ok(())
	}

	async fn open_with_git(self, path: &Path) -> Result<()> {
		let metadata = match std::fs::metadata(path) {
			Ok(metadata) => metadata,
			Err(e) => match self.mode {
				OpenMode::Force => {
					std::fs::File::create(path).with_context(|| format!("Failed to force-create file at '{}'.\n{e}", path.display()))?;
					std::fs::metadata(path).unwrap()
				}
				_ => eyre::bail!(
					"Failed to read metadata of file/directory at '{}', which means we do not have sufficient permissions or it does not exist",
					path.display()
				),
			},
		};
		let sp = match metadata.is_dir() {
			true => path.display(),
			false => path.parent().unwrap().display(),
		};

		Command::new("sh").arg("-c").arg(format!("git -C \"{sp}\" pull")).status().await.with_context(|| {
			format!("Failed to pull from Git repository at '{sp}'. Ensure a repository exists at this path or any of its parent directories and no merge conflicts are present.")
		})?;

		self.open_file(path)
			.await
			.with_context(|| format!("Failed to open file at '{}'. Use `OpenMode::Force` and ensure you have necessary permissions", path.display()))?;

		Command::new("sh")
			.arg("-c")
			.arg(format!("git -C \"{sp}\" add -A && git -C \"{sp}\" commit -m \".\" && git -C \"{sp}\" push"))
			.status()
			.await
			.with_context(|| format!("Failed to commit or push to Git repository at '{sp}'. Ensure you have the necessary permissions and the repository is correctly configured."))?;

		Ok(())
	}
}

/// Convenience function: opens file with default settings
pub async fn open<P: AsRef<Path>>(path: P) -> Result<()> {
	Client::default().open(path).await
}

/// Convenience function: opens file with default settings (blocking)
pub fn open_blocking<P: AsRef<Path>>(path: P) -> Result<()> {
	tokio::runtime::Runtime::new().unwrap().block_on(open(path))
}
