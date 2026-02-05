#![allow(dead_code)]
use async_trait::async_trait;

#[async_trait]
// Should trigger DE0504 - Client trait `UsersInfoClient2` ends with digit but missing V prefix
pub trait UsersInfoClient2: Send + Sync {
    async fn get_user(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0504 - Client trait `Client123` ends with digits but missing V prefix
pub trait Client123: Send + Sync {
    async fn calculate(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0504 - Client trait `UsersInfoClientV0` has zero version
pub trait UsersInfoClientV0: Send + Sync {
    async fn get_user_v0(&self) -> Result<(), ()>;
}

#[async_trait]
// Should trigger DE0504 - Client trait `UsersInfoClientV` has bare V without digits
pub trait UsersInfoClientV: Send + Sync {
    async fn list_users(&self) -> Result<(), ()>;
}

fn main() {}
