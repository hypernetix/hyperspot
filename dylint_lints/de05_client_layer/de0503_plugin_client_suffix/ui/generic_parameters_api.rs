#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `ThrPluginClient` suffix
pub trait ThrPluginApi<T>: Send + Sync {
    async fn get_data(&self) -> Result<T, ()>;
}

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `DataPluginClient` suffix
pub trait DataPluginApi<T, E>: Send + Sync
where
    T: Send + Sync,
    E: std::error::Error,
{
    async fn process(&self) -> Result<T, E>;
}

fn main() {}
