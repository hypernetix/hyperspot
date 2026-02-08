#![allow(dead_code)]
use async_trait::async_trait;

// Should NOT trigger - properly named with generics
#[async_trait]
pub trait ThrPluginClient<T>: Send + Sync {
    async fn get_data(&self) -> Result<T, ()>;
}

// Should NOT trigger - properly named with multiple generics
#[async_trait]
pub trait DataPluginClient<T, E>: Send + Sync
where
    T: Send + Sync,
    E: std::error::Error,
{
    async fn process(&self) -> Result<T, E>;
}

fn main() {}
