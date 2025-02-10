use std::{
	env,
	ffi::{CStr, OsString},
	mem,
	os::unix::ffi::OsStringExt,
	path::PathBuf,
	ptr,
};

// https://github.com/rust-lang/rust/blob/2682b88c526d493edeb2d3f2df358f44db69b73f/library/std/src/sys/unix/os.rs#L595
/// Stolen 1:1 from `dirs` crate. They themselves have some problems with wasm, so preferrable over an import.
#[cfg(feature = "io")]
#[cfg(not(target_os = "redox"))]
pub fn home_dir() -> Option<PathBuf> {
	return env::var_os("HOME")
		.and_then(|h| if h.is_empty() { None } else { Some(h) })
		.or_else(|| unsafe { fallback() })
		.map(PathBuf::from);

	#[cfg(any(target_os = "android", target_os = "ios", target_os = "emscripten"))]
	unsafe fn fallback() -> Option<OsString> {
		None
	}
	#[cfg(not(any(target_os = "android", target_os = "ios", target_os = "emscripten")))]
	unsafe fn fallback() -> Option<OsString> {
		let amt = match unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) } {
			n if n < 0 => 512_usize,
			n => n as usize,
		};
		let mut buf = Vec::with_capacity(amt);
		let mut passwd: libc::passwd = unsafe { mem::zeroed() };
		let mut result = ptr::null_mut();
		match unsafe { libc::getpwuid_r(libc::getuid(), &mut passwd, buf.as_mut_ptr(), buf.capacity(), &mut result) } {
			0 if !result.is_null() => {
				let ptr = passwd.pw_dir as *const _;
				let bytes = unsafe { CStr::from_ptr(ptr) }.to_bytes();
				if bytes.is_empty() { None } else { Some(OsStringExt::from_vec(bytes.to_vec())) }
			}
			_ => None,
		}
	}
}
#[cfg(target_os = "redox")]
pub fn home_dir() -> Option<PathBuf> {
	unimplemented!()
}
