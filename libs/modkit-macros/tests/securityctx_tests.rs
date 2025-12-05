#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for SecurityCtx-aware API generation

use modkit_macros::generate_clients;
use modkit_security::SecurityCtx;
use tonic::transport::Channel;

// Mock tonic client for testing
#[derive(Clone)]
pub struct MockSecuredClient<T> {
    _inner: T,
}

impl MockSecuredClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn get_user(
        &mut self,
        _req: tonic::Request<GetUserRequest>,
    ) -> Result<tonic::Response<UserResponse>, tonic::Status> {
        unimplemented!("mock")
    }

    pub async fn list_users(
        &mut self,
        _req: tonic::Request<ListUsersRequest>,
    ) -> Result<tonic::Response<UserListResponse>, tonic::Status> {
        unimplemented!("mock")
    }
}

// Domain types
#[derive(Clone, Debug)]
pub struct GetUserRequest {
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct ListUsersRequest {
    pub limit: usize,
}

#[derive(Clone, Debug)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
}

// Error type that implements From<tonic::Status>
#[derive(Debug)]
pub struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}

impl std::error::Error for TestError {}

impl From<tonic::Status> for TestError {
    fn from(status: tonic::Status) -> Self {
        TestError(format!("gRPC error: {}", status))
    }
}

// Secured API with SecurityCtx
#[generate_clients(grpc_client = "crate::MockSecuredClient<tonic::transport::Channel>")]
#[async_trait::async_trait]
pub trait SecuredApi: Send + Sync {
    async fn get_user(
        &self,
        ctx: &SecurityCtx,
        req: GetUserRequest,
    ) -> Result<UserResponse, TestError>;

    async fn list_users(
        &self,
        ctx: &SecurityCtx,
        req: ListUsersRequest,
    ) -> Result<UserListResponse, TestError>;
}

#[test]
fn test_secured_api_generates_grpc_client() {
    // Compile-time test: verify the generated struct exists
    let _type_check = |_: SecuredApiGrpcClient| {};
}

#[test]
fn test_secured_api_client_has_connect_method() {
    // Verify connect method signature exists
    let _fn_check: fn(String) -> _ = SecuredApiGrpcClient::connect;
}

#[test]
fn test_secured_api_client_has_from_channel_method() {
    // Verify from_channel method exists
    let _fn_check: fn(Channel) -> SecuredApiGrpcClient = SecuredApiGrpcClient::from_channel;
}

#[tokio::test]
async fn test_secured_api_client_connect_fails_without_server() {
    // Runtime test: connection fails gracefully when no server is available
    let result = SecuredApiGrpcClient::connect("http://[::1]:50051").await;
    assert!(result.is_err(), "Should fail to connect without a server");
}

// Unsecured API without SecurityCtx (for system services)
#[generate_clients(grpc_client = "crate::MockUnsecuredClient<tonic::transport::Channel>")]
#[async_trait::async_trait]
pub trait UnsecuredApi: Send + Sync {
    async fn ping(&self, msg: String) -> Result<String, TestError>;
}

#[derive(Clone)]
pub struct MockUnsecuredClient<T> {
    _inner: T,
}

impl MockUnsecuredClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn ping(
        &mut self,
        _req: tonic::Request<String>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        unimplemented!("mock")
    }
}

#[test]
fn test_unsecured_api_generates_grpc_client() {
    // Compile-time test: verify APIs without SecurityCtx still work
    let _type_check = |_: UnsecuredApiGrpcClient| {};
}
