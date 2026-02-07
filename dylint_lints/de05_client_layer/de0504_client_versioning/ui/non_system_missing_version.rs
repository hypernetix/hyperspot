#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0504 - Client trait `UsersInfoClient` in non-system module must have a version suffix
pub trait UsersInfoClient: Send + Sync {
    async fn get_user(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0504 - Client trait `CalculatorPluginClient` in non-system module must have a version suffix
pub trait CalculatorPluginClient: Send + Sync {
    async fn calculate(&self) -> Result<(), ()>;
}

fn main() {}
