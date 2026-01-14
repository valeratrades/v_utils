use std::{
	path::{Path, PathBuf},
	str::FromStr,
};

use eyre::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, de};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, derive_new::new)]
pub struct ExpandedPath(pub PathBuf);
impl ExpandedPath {
	pub fn inner(self) -> PathBuf {
		self.0
	}

	pub fn display(&self) -> std::path::Display<'_> {
		self.0.display()
	}

	pub fn parent(&self) -> Option<ExpandedPath> {
		self.0.parent().map(|p| ExpandedPath(p.to_path_buf()))
	}

	pub fn join<P: AsRef<Path>>(&self, path: P) -> ExpandedPath {
		ExpandedPath(self.0.join(path))
	}
}

impl<'de> Deserialize<'de> for ExpandedPath {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>, {
		let s = String::deserialize(deserializer)?;
		FromStr::from_str(&s).map_err(de::Error::custom)
	}
}
impl FromStr for ExpandedPath {
	type Err = eyre::Report;

	fn from_str(s: &str) -> Result<Self> {
		let path_buf = match s.starts_with("~") {
			true => {
				let home_dir = dirs::home_dir().ok_or_else(|| Error::msg("Failed to determine user's home directory"))?;

				match s.len() {
					l if l < 2 => Ok(home_dir),
					l if l > 2 => Ok(home_dir.join(&s[2..])),
					_ => Err(Error::msg("Incorrect Path")),
				}
			}
			false => Ok(PathBuf::from(s)),
		}?;

		Ok(ExpandedPath(path_buf))
	}
}

impl From<&str> for ExpandedPath {
	fn from(s: &str) -> Self {
		ExpandedPath::from_str(s).unwrap()
	}
}
impl From<String> for ExpandedPath {
	fn from(s: String) -> Self {
		ExpandedPath::from_str(&s).unwrap()
	}
}
impl From<PathBuf> for ExpandedPath {
	fn from(p: PathBuf) -> Self {
		ExpandedPath(p)
	}
}

impl std::fmt::Display for ExpandedPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.display().fmt(f)
	}
}
impl AsRef<Path> for ExpandedPath {
	fn as_ref(&self) -> &Path {
		self.0.as_ref()
	}
}
impl std::ops::Deref for ExpandedPath {
	type Target = PathBuf;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
