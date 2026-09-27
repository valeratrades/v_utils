/// The RFC 6238 code an authenticator app shows right now for `secret`, the base32 key a site
/// hands out when 2FA is set up (grouped, lowercase and padded forms all accepted).
pub fn totp(secret: &str, digits: u32) -> Result<String, data_encoding::DecodeError> {
	let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock after 1970").as_secs();
	at(secret, digits, now)
}
fn at(secret: &str, digits: u32, unix: u64) -> Result<String, data_encoding::DecodeError> {
	assert!((4..=10).contains(&digits), "TOTP is 4 to 10 digits, got {digits}");
	let key: String = secret.chars().filter(|c| !c.is_whitespace() && *c != '=').map(|c| c.to_ascii_uppercase()).collect();
	let key = data_encoding::BASE32_NOPAD.decode(key.as_bytes())?;
	Ok(totp_lite::totp_custom::<totp_lite::Sha1>(totp_lite::DEFAULT_STEP, digits, &key, unix))
}

#[cfg(test)]
mod tests {
	/// RFC 6238 appendix B, SHA1, key "12345678901234567890", as an authenticator would display it.
	#[test]
	fn rfc6238_vector() {
		assert_eq!(super::at("gezd gnbv gy3t qojq gezd gnbv gy3t qojq", 8, 59).unwrap(), "94287082");
		assert_eq!(super::at("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", 8, 1111111109).unwrap(), "07081804");
	}
}
