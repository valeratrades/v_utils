use std::{path::Path, process::Command};

use eyre::{Result, WrapErr, eyre};
use tokio::sync::oneshot;

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
	/// Mock mode for testing - waits for signal then returns without opening anything
	Mock(oneshot::Receiver<()>),
}

/// Builder for opening files with various options.
///
/// # Examples
/// ```ignore
/// use v_utils::io::file_open::Client;
///
/// // Simple open
/// Client::default().open(&path)?;
///
/// // Open with git sync
/// Client::default().git(true).open(&path)?;
///
/// // Force create and open
/// Client::default().mode(OpenMode::Force).open(&path)?;
///
/// // Open in pager with git sync
/// Client::default().git(true).mode(OpenMode::Pager).open(&path)?;
/// ```
#[derive(Debug, Default)]
pub struct Client {
	git: bool,
	mode: OpenMode,
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

	/// Open the file at the given path
	pub fn open<P: AsRef<Path>>(self, path: P) -> Result<()> {
		let path = path.as_ref();
		if self.git { self.open_with_git(path) } else { self.open_file(path) }
	}

	fn open_file(self, path: &Path) -> Result<()> {
		let p = path.display();
		match self.mode {
			OpenMode::Normal => {
				if !path.exists() {
					return Err(eyre!("File does not exist"));
				}
				Command::new("sh")
					.arg("-c")
					.arg(format!("$EDITOR {p}"))
					.status()
					.map_err(|_| eyre!("$EDITOR env variable is not defined"))?;
			}
			OpenMode::Force => {
				Command::new("sh")
					.arg("-c")
					.arg(format!("$EDITOR {p}"))
					.status()
					.map_err(|_| eyre!("$EDITOR env variable is not defined or permission lacking to create the file: {p}"))?;
			}
			OpenMode::Pager => {
				if !path.exists() {
					return Err(eyre!("File does not exist"));
				}
				Command::new("sh").arg("-c").arg(format!("less {p}")).status()?;
			}
			// Only works with nvim as I can't be bothered to look up "readonly" flag for all editors
			OpenMode::Read => {
				if !path.exists() {
					return Err(eyre!("File does not exist"));
				}
				Command::new("sh")
					.arg("-c")
					.arg(format!("nvim -R {p}"))
					.status()
					.map_err(|_| eyre!("nvim is not found in path"))?;
			}
			OpenMode::Mock(rx) => {
				if !path.exists() {
					return Err(eyre!("File does not exist"));
				}
				// Block until signal is received (or sender is dropped)
				let _ = rx.blocking_recv();
			}
		}

		Ok(())
	}

	fn open_with_git(self, path: &Path) -> Result<()> {
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

		Command::new("sh").arg("-c").arg(format!("git -C \"{sp}\" pull")).status().with_context(|| {
			format!(
				"Failed to pull from Git repository at '{}'. Ensure a repository exists at this path or any of its parent directories and no merge conflicts are present.",
				sp
			)
		})?;

		self.open_file(path)
			.with_context(|| format!("Failed to open file at '{}'. Use `OpenMode::Force` and ensure you have necessary permissions", path.display()))?;

		Command::new("sh")
			.arg("-c")
			.arg(format!("git -C \"{sp}\" add -A && git -C \"{sp}\" commit -m \".\" && git -C \"{sp}\" push"))
			.status()
			.with_context(|| {
				format!(
					"Failed to commit or push to Git repository at '{}'. Ensure you have the necessary permissions and the repository is correctly configured.",
					sp
				)
			})?;

		Ok(())
	}
}

/// Convenience function: opens file with default settings
pub fn open<P: AsRef<Path>>(path: P) -> Result<()> {
	Client::default().open(path)
}
