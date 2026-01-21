//! gRPC server implementation for `DirectoryService`
//!
//! This module provides the gRPC service implementation for Directory Service.

use std::sync::Arc;
use tonic::{Request, Response, Status};

use module_orchestrator_grpc::{
    DeregisterInstanceRequest, DirectoryService, DirectoryServiceServer, HeartbeatRequest,
    InstanceInfo, ListInstancesRequest, ListInstancesResponse, RegisterInstanceRequest,
    ResolveGrpcServiceRequest, ResolveGrpcServiceResponse,
};
use module_orchestrator_sdk::{DirectoryClient, RegisterInstanceInfo, ServiceEndpoint};

/// gRPC service implementation of Directory Service
#[derive(Clone)]
pub struct DirectoryServiceImpl {
    api: Arc<dyn DirectoryClient>,
}

impl DirectoryServiceImpl {
    pub fn new(api: Arc<dyn DirectoryClient>) -> Self {
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

    async fn register_instance(
        &self,
        request: Request<RegisterInstanceRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        // Parse endpoints from GrpcServiceEndpoint messages
        let grpc_services = req
            .grpc_services
            .into_iter()
            .map(|svc| (svc.service_name, ServiceEndpoint::new(svc.endpoint_uri)))
            .collect();

        let info = RegisterInstanceInfo {
            module: req.module_name,
            instance_id: req.instance_id,
            grpc_services,
            version: if req.version.is_empty() {
                None
            } else {
                Some(req.version)
            },
        };

        self.api
            .register_instance(info)
            .await
            .map_err(|e| Status::internal(format!("Failed to register instance: {e}")))?;

        Ok(Response::new(()))
    }

    async fn deregister_instance(
        &self,
        request: Request<DeregisterInstanceRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        self.api
            .deregister_instance(&req.module_name, &req.instance_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to deregister instance: {e}")))?;

        Ok(Response::new(()))
    }

    async fn heartbeat(&self, request: Request<HeartbeatRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        self.api
            .send_heartbeat(&req.module_name, &req.instance_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to send heartbeat: {e}")))?;

        Ok(Response::new(()))
    }
}

/// Create a `DirectoryService` server with the given API implementation
pub fn make_directory_service(
    api: Arc<dyn DirectoryClient>,
) -> DirectoryServiceServer<DirectoryServiceImpl> {
    DirectoryServiceServer::new(DirectoryServiceImpl::new(api))
}
