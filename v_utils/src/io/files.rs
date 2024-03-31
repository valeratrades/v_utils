use anyhow::{anyhow, Context, Result};
use std::{path::PathBuf, process::Command};

pub enum OpenMode {
	Normal,
	Force,
	Readonly,
}

pub fn open_with_mode(path: &PathBuf, mode: OpenMode) -> Result<()> {
	let p = path.display();
	match mode {
		OpenMode::Normal => {
			if !path.exists() {
				return Err(anyhow!("File does not exist"));
			}
			Command::new("sh")
				.arg("-c")
				.arg(format!("$EDITOR {p}"))
				.status()
				.map_err(|_| anyhow!("$EDITOR env variable is not defined"))?;
		}
		OpenMode::Force => {
			Command::new("sh")
				.arg("-c")
				.arg(format!("$EDITOR {p}"))
				.status()
				.map_err(|_| anyhow!("$EDITOR env variable is not defined or permission lacking to create the file: {p}"))?;
		}
		OpenMode::Readonly => {
			if !path.exists() {
				return Err(anyhow!("File does not exist"));
			}
			Command::new("sh").arg("-c").arg(format!("less {p}")).status()?;
		}
	}

	Ok(())
}

/// Wrapper around `open_with_mode` that syncs with git. If `open_mode` provided, it will open the file in-between.
pub fn sync_file_with_git(path: &PathBuf, open_mode: Option<OpenMode>) -> Result<()> {
	let p = path.display();
	Command::new("sh")
		.arg("-c")
		.arg(format!("git -C \"{p}\" pull"))
		.status()
		.with_context(|| format!("Failed to pull from Git repository at '{}'. Ensure a repository exists at this path or any of its parent directories and no merge conflicts are present.", p))?;

	if let Some(open_mode) = open_mode {
		open_with_mode(path, open_mode).with_context(|| {
			format!(
				"Failed to open file at '{}'. Use `OpenMode::Force` and ensure you have necessary permissions",
				path.display()
			)
		})?;
	}

	Command::new("sh")
		.arg("-c")
		.arg(format!("git -C \"{p}\" add -A && git -C \"{p}\" commit -m \".\" && git -C \"{p}\" push"))
		.status()
		.with_context(|| {
			format!("Failed to commit or push to Git repository at '{}'. Ensure you have the necessary permissions and the repository is correctly configured.", p)
		})?;

	Ok(())
}

/// Convenience function.
pub fn open(path: &PathBuf) -> Result<()> {
	open_with_mode(path, OpenMode::Normal)
}
