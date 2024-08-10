use std::future::Future;
use std::pin::Pin;
pub use tokio::task::JoinSet;

pub trait JoinSetExt {
	fn join_all(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: 'static> JoinSetExt for JoinSet<T> {
	fn join_all(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
		Box::pin(async move { while (self.join_next().await).is_some() {} })
	}
}
