use axum::{
    Extension,
    extract::{Path, Query},
};
use modkit::api::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use super::dto::{NodeDto, NodeSysCapDto, NodeSysInfoDto};
use crate::domain::service::Service;

#[derive(Debug, Deserialize)]
pub struct DetailsQuery {
    #[serde(default)]
    pub details: bool,
    #[serde(default)]
    pub force_refresh: bool,
}

#[derive(Debug, Deserialize)]
pub struct SysCapQuery {
    /// Force refresh syscap, ignoring cache
    #[serde(default)]
    pub force_refresh: bool,
}

/// List all nodes
pub async fn list_nodes(
    Extension(svc): Extension<Arc<Service>>,
    Query(query): Query<DetailsQuery>,
) -> ApiResult<Json<Vec<NodeDto>>> {
    let nodes = svc.list_nodes();

    if query.details {
        // Include sysinfo and syscap for each node
        let mut detailed_nodes = Vec::new();
        for node in nodes {
            let node_id = node.id;
            let sysinfo = svc.get_node_sysinfo(node_id).ok().map(Into::into);
            let syscap = svc
                .get_node_syscap(node_id, query.force_refresh)
                .ok()
                .map(Into::into);

            let mut node_dto: NodeDto = node.into();
            node_dto.sysinfo = sysinfo;
            node_dto.syscap = syscap;
            detailed_nodes.push(node_dto);
        }
        Ok(Json(detailed_nodes))
    } else {
        Ok(Json(nodes.into_iter().map(Into::into).collect()))
    }
}

/// Get a node by ID
pub async fn get_node(
    Extension(svc): Extension<Arc<Service>>,
    Path(id): Path<uuid::Uuid>,
    Query(query): Query<DetailsQuery>,
) -> ApiResult<Json<NodeDto>> {
    let node = svc.get_node(id)?;

    if query.details {
        let sysinfo = svc.get_node_sysinfo(id).ok().map(Into::into);
        let syscap = svc
            .get_node_syscap(id, query.force_refresh)
            .ok()
            .map(Into::into);

        let mut node_dto: NodeDto = node.into();
        node_dto.sysinfo = sysinfo;
        node_dto.syscap = syscap;
        Ok(Json(node_dto))
    } else {
        Ok(Json(node.into()))
    }
}

/// Get system information for a node
pub async fn get_node_sysinfo(
    Extension(svc): Extension<Arc<Service>>,
    Path(node_id): Path<uuid::Uuid>,
) -> ApiResult<Json<NodeSysInfoDto>> {
    let sysinfo = svc.get_node_sysinfo(node_id)?;
    Ok(Json(sysinfo.into()))
}

/// Get system capabilities for a node
pub async fn get_node_syscap(
    Extension(svc): Extension<Arc<Service>>,
    Path(node_id): Path<uuid::Uuid>,
    Query(query): Query<SysCapQuery>,
) -> ApiResult<Json<NodeSysCapDto>> {
    let syscap = svc.get_node_syscap(node_id, query.force_refresh)?;
    Ok(Json(syscap.into()))
}
