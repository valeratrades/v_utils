//! Memory a pod holds only while it needs it: devops' memory-lease broker resizes the pod to its
//! declared burst for as long as a [`Lease`] lives, and back to its base once none does.
use eyre::{Result, bail, eyre};
use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::TcpStream,
};

/// The burst lasts until this is dropped, or the process dies.
#[must_use = "the burst ends when the lease is dropped"]
pub struct Lease(Option<BufReader<TcpStream>>);

impl Lease {
	/// Waits until kubelet runs this pod at its burst size. Fails if the node can never fit it.
	///
	/// Without `MEMORY_LEASE_URL` the process runs outside any broker's cluster, with no limit to raise, and the lease is inert.
	pub async fn acquire() -> Result<Self> {
		let Ok(url) = std::env::var("MEMORY_LEASE_URL") else { return Ok(Self(None)) };
		let rest = url.strip_prefix("http://").ok_or_else(|| eyre!("MEMORY_LEASE_URL={url}: not an http:// URL"))?;
		let (authority, path) = rest.split_once('/').ok_or_else(|| eyre!("MEMORY_LEASE_URL={url}: no path"))?;
		let mut stream = TcpStream::connect(authority).await?;
		// HTTP/1.0, so the body is plain lines until close rather than chunks
		stream.write_all(format!("GET /{path} HTTP/1.0\r\nHost: {authority}\r\n\r\n").as_bytes()).await?;
		let mut r = BufReader::new(stream);
		let mut status = String::new();
		r.read_line(&mut status).await?;
		let mut line = String::new();
		while r.read_line(&mut line).await? > 2 {
			line.clear();
		}
		if status.split_whitespace().nth(1) != Some("200") {
			let mut body = String::new();
			r.read_to_string(&mut body).await?;
			bail!("{url}: {} {}", status.trim_end(), body.trim_end());
		}
		loop {
			line.clear();
			if r.read_line(&mut line).await? == 0 {
				bail!("{url} closed before granting the burst");
			}
			match line.trim_end() {
				"granted" => return Ok(Self(Some(r))),
				"" => {}
				l if l.starts_with("infeasible") => {
					bail!("{url}: {l}");
				}
				l => tracing::info!("memory lease: {l}"),
			}
		}
	}
}

impl Drop for Lease {
	fn drop(&mut self) {
		drop(self.0.take()); // the closed connection is the broker's only signal
	}
}
