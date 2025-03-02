//Q: not sure this is even desirable. Think of eyre, for example: trying to expose eyre::{Result, bail, eyre} like this will lead to conflicts between color_eyre::eyre and eyre
// I guess I could split the preludes to `{client,library}-side`s
//Q: color-eyre/eyre argument is nivilated, does anything else prevent me from having a joined prelude?

pub use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
	fmt::Write as _,
	pin::Pin,
	str::FromStr as _,
	sync::{Arc, Mutex, RwLock},
};

pub use chrono::{DateTime, Utc};
pub use eyre::{OptionExt as _, Report, Result, WrapErr as _, bail, eyre};
pub use futures::future::join_all;
pub use serde::{
	Deserialize, Serialize, Serializer,
	de::{DeserializeOwned, Deserializer},
};
pub use serde_json::{Value, json};
pub use thiserror::Error;
// not yet used in this lib, don't want to import just for thsi
//use serde_with::{serde_as, DisplayFromStr};
pub use tracing::{Span, debug, error, field::Empty, info, instrument, trace, warn};

pub use crate::{clientside, io::ExpandedPath, other::*, trades::*};

#[deprecated(note = "Use main `prelude` instead")]
pub mod clientside {
	pub use super::*;
	pub use crate::clientside;
}

#[deprecated(note = "Use main `prelude` instead")]
pub mod libside {
	pub use super::*;
}
