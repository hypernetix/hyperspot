#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `ThrPluginClientV2` suffix
pub trait ThrPluginApi2: Send + Sync {
    async fn get_data(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `DataPluginClient` suffix
pub trait DataPluginApiV: Send + Sync {
    async fn process(&self) -> Result<(), ()>;
}

fn main() {}
