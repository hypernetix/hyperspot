#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0504 - Client trait `UsersInfoClient` with generic parameter missing version suffix
pub trait UsersInfoClient<T>: Send + Sync {
    async fn get_user(&self) -> Result<T, ()>;
}

#[async_trait]
// Should trigger DE0504 - Client trait `CalculatorClient` with multiple generic parameters missing version suffix
pub trait CalculatorClient<T, E>: Send + Sync
where
    T: Send + Sync,
    E: std::error::Error,
{
    async fn calculate(&self) -> Result<T, E>;
}

#[async_trait]
// Should NOT trigger - properly versioned with generics
pub trait DataClientV1<T>: Send + Sync {
    async fn get_data(&self) -> Result<T, ()>;
}

fn main() {}
