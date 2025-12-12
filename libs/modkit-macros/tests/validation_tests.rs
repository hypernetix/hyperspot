#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unused_async,
    clippy::no_effect_underscore_binding
)]

//! Validation and error handling tests for client macros (gRPC only)

use tonic::transport::Channel;

// Valid API for testing (compile-time validation)
#[modkit_macros::generate_clients(
    grpc_client = "crate::MockValidClient<tonic::transport::Channel>"
)]
#[async_trait::async_trait]
pub trait ValidApi: Send + Sync {
    async fn operation(&self, req: String) -> Result<String, anyhow::Error>;
}

#[derive(Clone)]
pub struct MockValidClient<T> {
    _inner: T,
}

impl MockValidClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn operation(
        &mut self,
        _req: tonic::Request<String>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        unimplemented!("mock")
    }
}

#[test]
fn test_valid_api() {
    // Compile-time test: valid API generates successfully
    let _type_check = |_: ValidApiGrpcClient| {};
}

// Test: API with explicit Result<T, E> works
#[modkit_macros::generate_clients(
    grpc_client = "crate::MockExplicitClient<tonic::transport::Channel>"
)]
#[async_trait::async_trait]
pub trait ExplicitResultApi: Send + Sync {
    async fn get_data(&self, req: String) -> Result<String, anyhow::Error>;
}

#[derive(Clone)]
pub struct MockExplicitClient<T> {
    _inner: T,
}

impl MockExplicitClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn get_data(
        &mut self,
        _req: tonic::Request<String>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        unimplemented!("mock")
    }
}

#[test]
fn test_explicit_result_type() {
    // Compile-time test: explicit Result<T, E> works
    let _type_check = |_: ExplicitResultApiGrpcClient| {};
}

// Test: API with custom error type works
#[derive(Debug)]
pub struct CustomError(String);

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomError: {}", self.0)
    }
}

impl std::error::Error for CustomError {}

impl From<tonic::Status> for CustomError {
    fn from(status: tonic::Status) -> Self {
        CustomError(format!("gRPC error: {}", status))
    }
}

#[modkit_macros::generate_clients(
    grpc_client = "crate::MockCustomErrorClient<tonic::transport::Channel>"
)]
#[async_trait::async_trait]
pub trait CustomErrorApi: Send + Sync {
    async fn operation(&self, req: String) -> Result<String, CustomError>;
}

#[derive(Clone)]
pub struct MockCustomErrorClient<T> {
    _inner: T,
}

impl MockCustomErrorClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn operation(
        &mut self,
        _req: tonic::Request<String>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        unimplemented!("mock")
    }
}

#[test]
fn test_custom_error_type() {
    // Compile-time test: custom error types work
    let _type_check = |_: CustomErrorApiGrpcClient| {};
}

// Test: Multiple methods with different return types
#[modkit_macros::generate_clients(
    grpc_client = "crate::MockMultiMethodClient<tonic::transport::Channel>"
)]
#[async_trait::async_trait]
pub trait MultiMethodApi: Send + Sync {
    async fn get_string(&self, req: String) -> Result<String, anyhow::Error>;
    async fn get_number(&self, req: i32) -> Result<i32, anyhow::Error>;
    async fn get_vec(&self, req: Vec<String>) -> Result<Vec<String>, anyhow::Error>;
}

#[derive(Clone)]
pub struct MockMultiMethodClient<T> {
    _inner: T,
}

impl MockMultiMethodClient<Channel> {
    pub fn new(inner: Channel) -> Self {
        Self { _inner: inner }
    }

    pub async fn get_string(
        &mut self,
        _req: tonic::Request<String>,
    ) -> Result<tonic::Response<String>, tonic::Status> {
        unimplemented!("mock")
    }

    pub async fn get_number(
        &mut self,
        _req: tonic::Request<i32>,
    ) -> Result<tonic::Response<i32>, tonic::Status> {
        unimplemented!("mock")
    }

    pub async fn get_vec(
        &mut self,
        _req: tonic::Request<Vec<String>>,
    ) -> Result<tonic::Response<Vec<String>>, tonic::Status> {
        unimplemented!("mock")
    }
}

#[test]
fn test_multiple_methods() {
    // Compile-time test: multiple methods generate correctly
    let _type_check = |_: MultiMethodApiGrpcClient| {};
}

// Test: API with only one method
#[modkit_macros::generate_clients(
    grpc_client = "crate::MockSingleClient<tonic::transport::Channel>"
)]
#[async_trait::async_trait]
pub trait SingleMethodApi: Send + Sync {
    async fn do_something(&self, req: String) -> Result<String, anyhow::Error>;
}

#[derive(Clone)]
pub struct MockSingleClient<T> {
    _inner: T,
}

impl MockSingleClient<Channel> {
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
fn test_single_method_api() {
    // Compile-time test: single-method API works
    let _type_check = |_: SingleMethodApiGrpcClient| {};
}

#[test]
fn validation_tests_compile() {
    // If we got here, all validation tests passed at compile time
}
