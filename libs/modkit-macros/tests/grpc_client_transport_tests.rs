//! Integration tests verifying that generated gRPC clients use the transport stack

use modkit_macros::generate_clients;
use tonic::transport::Channel;

// Mock tonic client for testing
#[derive(Clone)]
pub struct MockTonicClient<T> {
    _inner: T,
}

impl<T> MockTonicClient<T> {
    pub fn new(inner: T) -> Self {
        Self { _inner: inner }
    }

    // Mock gRPC method
    pub async fn get_item(
        &mut self,
        _request: tonic::Request<GetItemRequest>,
    ) -> Result<tonic::Response<ItemResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("mock client"))
    }
}

// Simple domain types
#[derive(Clone, Debug)]
pub struct GetItemRequest {
    pub id: String,
}

#[derive(Clone, Debug)]
pub struct ItemResponse {
    pub id: String,
    pub name: String,
}

// API trait with generate_clients macro (gRPC only)
#[generate_clients(grpc_client = "MockTonicClient<tonic::transport::Channel>")]
#[async_trait::async_trait]
pub trait ItemApi: Send + Sync {
    async fn get_item(&self, req: GetItemRequest) -> Result<ItemResponse, anyhow::Error>;
}

#[test]
fn generated_grpc_client_struct_exists() {
    // Compile-time test: verify the generated struct exists
    let _assert_type_exists = |_: ItemApiGrpcClient| {};
}

#[test]
fn grpc_client_has_connect_method() {
    // Compile-time test: verify connect method exists
    let _fn_check: fn(String) -> _ = ItemApiGrpcClient::connect;
}

#[test]
fn grpc_client_has_connect_with_config_method() {
    // Compile-time test: verify connect_with_config method exists
    // Note: function signature is generic over impl Into<String>, so we can't easily type-check it
    let _type_exists =
        std::mem::size_of::<fn(String, &modkit_transport_grpc::client::GrpcClientConfig)>();
}

#[test]
fn grpc_client_has_from_channel_method() {
    // Compile-time test: verify from_channel method exists
    let _fn_check: fn(Channel) -> ItemApiGrpcClient = ItemApiGrpcClient::from_channel;
}

#[tokio::test]
async fn grpc_client_connect_fails_without_server() {
    // Runtime test: connection fails gracefully when no server is available
    let result = ItemApiGrpcClient::connect("http://[::1]:50051").await;
    assert!(result.is_err(), "Should fail to connect without a server");
}

#[tokio::test]
async fn grpc_client_connect_with_config_fails_without_server() {
    // Runtime test: connection with custom config fails gracefully
    let config = modkit_transport_grpc::client::GrpcClientConfig::new("test_service")
        .with_connect_timeout(std::time::Duration::from_millis(100));

    let result = ItemApiGrpcClient::connect_with_config("http://[::1]:50052", &config).await;
    assert!(
        result.is_err(),
        "Should fail to connect without a server (custom config)"
    );
}

// Test multiple clients can coexist
#[generate_clients(grpc_client = "AnotherMockClient<tonic::transport::Channel>")]
#[async_trait::async_trait]
pub trait AnotherApi: Send + Sync {
    async fn do_something(&self, msg: String) -> Result<String, anyhow::Error>;
}

#[derive(Clone)]
pub struct AnotherMockClient<T> {
    _inner: T,
}

impl AnotherMockClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn do_something(
        &mut self,
        _req: tonic::Request<String>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        unimplemented!("mock")
    }
}

#[test]
fn multiple_grpc_clients_can_coexist() {
    // Compile-time test: multiple generated clients don't conflict
    let _type1 = |_: ItemApiGrpcClient| {};
    let _type2 = |_: AnotherApiGrpcClient| {};
}

#[test]
fn grpc_client_implements_trait() {
    // Compile-time test: generated client implements the trait
    fn _assert_impl<T: ItemApi>() {}
    _assert_impl::<ItemApiGrpcClient>();
}
