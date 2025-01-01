pub mod eyre;
pub use eyre::*;

pub mod snapshots;
pub use snapshots::*;

pub mod serde;
pub use serde::*;

#[cfg(feature = "tracing")]
pub mod tracing;
#[cfg(feature = "tracing")]
pub use tracing::*;

//use std::collections::HashMap;
//use thiserror::Error;
//
//#[derive(Error, Debug)]
//#[error("Columns in provided data are not aligned")]
//pub struct NotAlignedError;
//
////TODO!: make the now name String be a str-vec
///// Fuck polars, this is my way to represent alignment property
//#[derive(Clone, Debug, Default)]
////TODO!: write the operations, right now there is no protection from updates that would break alignment property
//pub struct Df<T: Clone + std::fmt::Debug, I: Clone + std::fmt::Debug>(pub HashMap<String, Vec<T>>);
//
//impl<T: Clone + std::fmt::Debug> Df<T> {
//	pub fn new(map: HashMap<String, Vec<T>>) -> Result<Self, NotAlignedError> {
//		let first_len = match map.iter().next() {
//			Some((_, v)) => v.len(),
//			None => return Ok(Self(HashMap::new())),
//		};
//		for col in map.iter() {
//			if col.1.len() != first_len {
//				return Err(NotAlignedError);
//			}
//		}
//
//		Ok(Self(map))
//	}
//}
