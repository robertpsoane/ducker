use crate::traits::Callback;
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct EmptyCallable {}

impl Default for EmptyCallable {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyCallable {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Callback for EmptyCallable {
    async fn call(&self) -> Result<()> {
        Ok(())
    }
}
