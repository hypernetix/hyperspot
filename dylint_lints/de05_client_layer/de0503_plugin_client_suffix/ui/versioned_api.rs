#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `ThrPluginClientV1` suffix
pub trait ThrPluginApiV1: Send + Sync {
    async fn get_data(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `DataPluginClientV2` suffix
pub trait DataPluginApiV2: Send + Sync {
    async fn process(&self) -> Result<(), ()>;
}

fn main() {}
