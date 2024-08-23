use eyre::{eyre, Result, WrapErr};
use std::{
	path::{Path, PathBuf},
	process::Command,
};

pub enum OpenMode {
	Normal,
	Force,
	Read,
	Pager,
}

pub fn open_with_mode(path: &Path, mode: OpenMode) -> Result<()> {
	let p = path.display();
	match mode {
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
	}

	Ok(())
}

/// Wrapper around `open_with_mode` that syncs with git. If `open_mode` provided, it will open the file in-between.
pub fn sync_file_with_git(path: &PathBuf, open_mode: Option<OpenMode>) -> Result<()> {
	let metadata = match std::fs::metadata(path) {
		Ok(metadata) => metadata,
		Err(e) => match open_mode {
			Some(OpenMode::Force) => {
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

	Command::new("sh")
		.arg("-c")
		.arg(format!("git -C \"{sp}\" pull"))
		.status()
		.with_context(|| format!("Failed to pull from Git repository at '{}'. Ensure a repository exists at this path or any of its parent directories and no merge conflicts are present.", sp))?;

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
		.arg(format!("git -C \"{sp}\" add -A && git -C \"{sp}\" commit -m \".\" && git -C \"{sp}\" push"))
		.status()
		.with_context(|| {
			format!("Failed to commit or push to Git repository at '{}'. Ensure you have the necessary permissions and the repository is correctly configured.", sp)
		})?;

	Ok(())
}

/// Convenience function.
pub fn open(path: &Path) -> Result<()> {
	open_with_mode(path, OpenMode::Normal)
}
