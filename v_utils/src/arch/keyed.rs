use std::{fmt::Debug, hash::Hash};

pub trait Keyed {
	type Key: KeyBounds;
	fn keys(&self) -> MyKey<Self::Key>;

	fn id(&self) -> Self::Key {
		self.keys().id
	}
	fn parent(&self) -> Option<Self::Key> {
		self.keys().parent
	}
}
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, derive_new::new)]
pub struct MyKey<Key: KeyBounds> {
	pub id: Key,
	pub parent: Option<Key>,
}
pub trait KeyBounds: Eq + Hash + Copy + Debug + PartialEq + Default {}
impl<T: Eq + Hash + Copy + Debug + PartialEq + Default> KeyBounds for T {}
