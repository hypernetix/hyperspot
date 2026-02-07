#![allow(dead_code)]
use async_trait::async_trait;

// Should not trigger DE0504 - has V1 suffix
#[async_trait]
pub trait UsersInfoClientV1: Send + Sync {
    async fn get_user(&self) -> Result<(), ()>;
}

// Should not trigger DE0504 - has V2 suffix
#[async_trait]
pub trait CalculatorPluginClientV2: Send + Sync {
    async fn calculate(&self) -> Result<(), ()>;
}

// Should not trigger DE0504 - has V1 suffix
#[async_trait]
pub trait SimpleUserSettingsClientV1: Send + Sync {
    async fn get_settings(&self) -> Result<(), ()>;
}

fn main() {}
