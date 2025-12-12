//! gRPC server implementation for DirectoryService

use std::sync::Arc;
use tonic::{Request, Response, Status};

use modkit::DirectoryApi;

/// Generated protobuf types
#[allow(clippy::default_trait_access)]
pub mod proto {
    pub mod directory {
        pub mod v1 {
            tonic::include_proto!("modkit.directory.v1");
        }
    }
}

use proto::directory::v1::{
    directory_service_server::{DirectoryService, DirectoryServiceServer},
    InstanceInfo, ListInstancesRequest, ListInstancesResponse, ResolveGrpcServiceRequest,
    ResolveGrpcServiceResponse,
};

// Export the service name constant for use by the module
pub use proto::directory::v1::directory_service_server::SERVICE_NAME;

/// gRPC service implementation that wraps DirectoryApi
#[derive(Clone)]
pub struct DirectoryServiceImpl {
    api: Arc<dyn DirectoryApi>,
}

impl DirectoryServiceImpl {
    pub fn new(api: Arc<dyn DirectoryApi>) -> Self {
        Self { api }
    }
}

#[tonic::async_trait]
impl DirectoryService for DirectoryServiceImpl {
    async fn resolve_grpc_service(
        &self,
        request: Request<ResolveGrpcServiceRequest>,
    ) -> Result<Response<ResolveGrpcServiceResponse>, Status> {
        let service_name = request.into_inner().service_name;

        let endpoint = self
            .api
            .resolve_grpc_service(&service_name)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(ResolveGrpcServiceResponse {
            endpoint_uri: endpoint.uri,
        }))
    }

    async fn list_instances(
        &self,
        request: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let module_name = request.into_inner().module_name;

        let instances = self
            .api
            .list_instances(&module_name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let resp = ListInstancesResponse {
            instances: instances
                .into_iter()
                .map(|i| InstanceInfo {
                    module_name: i.module,
                    instance_id: i.instance_id,
                    endpoint_uri: i.endpoint.uri,
                    version: i.version.unwrap_or_default(),
                })
                .collect(),
        };

        Ok(Response::new(resp))
    }
}

/// Create a DirectoryService server with the given API implementation
pub fn make_directory_service(
    api: Arc<dyn DirectoryApi>,
) -> DirectoryServiceServer<DirectoryServiceImpl> {
    DirectoryServiceServer::new(DirectoryServiceImpl::new(api))
}
