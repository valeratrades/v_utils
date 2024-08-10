#![allow(async_fn_in_trait)]
pub use tokio::task::JoinSet;

pub trait JoinSetExt {
    async fn join_all(&mut self);
}

impl<T: 'static> JoinSetExt for JoinSet<T> {
    async fn join_all(&mut self) {
        while (self.join_next().await).is_some() {}
    }
}

