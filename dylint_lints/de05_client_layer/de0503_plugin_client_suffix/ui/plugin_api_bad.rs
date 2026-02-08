#![allow(dead_code)]

use async_trait::async_trait;

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `ThrPluginClient` suffix
pub trait ThrPluginApi: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0503 - plugin client traits should use `OagwPluginClient` suffix
pub trait OagwPluginApi: Send + Sync {
    async fn execute(&self) -> Result<(), ()>;
}

fn main() {}
