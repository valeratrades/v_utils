use anyhow::{Error, Result};
use dirs;
use serde::{de::Error as SerdeError, Deserialize, Deserializer};
use std::convert::AsRef;
use std::str::FromStr;
use std::{path::Path, path::PathBuf};

#[derive(Clone, Debug)]
pub struct ExpandedPath(pub PathBuf);
impl<'de> Deserialize<'de> for ExpandedPath {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let path = String::deserialize(deserializer)?;
		//TODO!!!!!!!!!: implement the D::Error::custom
		let _p = expand_tilde(&path).map_err(|e| SerdeError::custom(e.to_string()))?;
		Ok(ExpandedPath(_p))
	}
}
impl FromStr for ExpandedPath {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self> {
		Ok(ExpandedPath(expand_tilde(s)?))
	}
}
fn expand_tilde(path: &str) -> Result<PathBuf> {
	if path.starts_with("~") {
		let home_dir = dirs::home_dir().ok_or_else(|| Error::msg("Failed to determine user's home directory"))?;

		match path.len() {
			l if l < 2 => Ok(home_dir),
			l if l > 2 => Ok(home_dir.join(&path[2..])),
			_ => Err(Error::msg("Incorrect Path")),
		}
	} else {
		Ok(PathBuf::from(path))
	}
}
impl std::fmt::Display for ExpandedPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0.display())
	}
}
impl AsRef<Path> for ExpandedPath {
	fn as_ref(&self) -> &Path {
		self.0.as_ref()
	}
}

impl ExpandedPath {
	pub fn process(self) -> PathBuf {
		self.0
	}
}
