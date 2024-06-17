use std::fmt::Debug;

use async_trait::async_trait;

#[async_trait]
pub trait Callback: Debug + Send + Sync {
    async fn call(&self) {}
}
