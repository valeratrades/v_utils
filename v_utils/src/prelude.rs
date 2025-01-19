//Q: not sure this is even desirable. Think of eyre, for example: trying to expose eyre::{Result, bail, eyre} like this will lead to conflicts between color_eyre::eyre and eyre
// I guess I could split the preludes to `{client,library}-side`s

pub use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	pin::Pin,
	str::FromStr as _,
};

pub use futures::future::join_all;
pub use serde::{
	de::{DeserializeOwned, Deserializer},
	Deserialize, Serialize, Serializer,
};
pub use serde_json::{json, Value};
// not yet used in this lib, don't want to import just for thsi
//use serde_with::{serde_as, DisplayFromStr};
pub use tracing::{debug, error, info, instrument, trace, warn};

pub use crate::{io::ExpandedPath, other::*, trades::*};
pub mod clientside {
	pub use super::*;
	// don't want to import color_eyre just for this
	//pub use color_eyre::eyre::{bail, eyre, Result};
	pub use crate::clientside;
}

pub mod libside {
	pub use eyre::{bail, eyre, Result};

	pub use super::*;
}
