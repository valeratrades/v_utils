/// Produce pretty Display of itself. `root_indent` is used when print contains newlines.
pub trait PrettyPrint {
	fn pretty(&self, f: &mut std::fmt::Formatter<'_>, root_indent: u8) -> std::fmt::Result;
	fn to_string_pretty(&self, root_indent: u8) -> String {
		struct Wrapper<'a, T: ?Sized>(&'a T, u8);
		impl<T: PrettyPrint + ?Sized> std::fmt::Display for Wrapper<'_, T> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				self.0.pretty(f, self.1)
			}
		}
		Wrapper(self, root_indent).to_string()
	}
}
