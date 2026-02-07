#![allow(dead_code)]
use async_trait::async_trait;

// Should not trigger DE0504 - Client trait naming does not apply, base does not end with Client
#[async_trait]
pub trait ClientEventHandler: Send + Sync {
    async fn handle(&self) -> Result<(), ()>;
}

// Should not trigger DE0504 - Client trait naming does not apply, base does not end with Client
#[async_trait]
pub trait ClientConfiguration: Send + Sync {
    async fn configure(&self) -> Result<(), ()>;
}

// Should not trigger DE0504 - Client trait naming does not apply, base does not end with Client
#[async_trait]
pub trait ApiClientAdapter: Send + Sync {
    async fn adapt(&self) -> Result<(), ()>;
}

// Should not trigger DE0504 - Client trait naming does not apply, no Client suffix
#[async_trait]
pub trait DataProcessor: Send + Sync {
    async fn process(&self) -> Result<(), ()>;
}

fn main() {}
