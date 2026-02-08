#![allow(dead_code)]
use async_trait::async_trait;

// Should not trigger DE0503 - PluginClientV1 suffix is valid
#[async_trait]
pub trait TenantResolverPluginClientV1: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}

// Should not trigger DE0503 - ClientV1 suffix is valid
#[async_trait]
pub trait TenantResolverClientV1: Send + Sync {
    async fn list_tenants(&self) -> Result<(), ()>;
}

// Should not trigger DE0503 - PluginClientV2 suffix is valid
#[async_trait]
pub trait TenantResolverPluginClientV2: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}

fn main() {}
