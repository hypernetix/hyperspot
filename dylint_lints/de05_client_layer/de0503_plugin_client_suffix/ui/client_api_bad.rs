#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `SomeClient` suffix
pub trait SomeClientApi: Send + Sync {
    async fn get_data(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `ThrPluginClientV1` suffix
pub trait ThrPluginClientApiV1: Send + Sync {
    async fn resolve(&self) -> Result<(), ()>;
}

fn main() {}
