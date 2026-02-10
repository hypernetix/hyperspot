//! API traits for `tenant_resolver`.
//!
//! This module defines two separate API traits:
//! - `TenantResolverClientV1` - Public API exposed by the gateway to other modules
//! - `TenantResolverPluginClientV1` - Internal API implemented by plugins, called by the gateway

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;

use crate::{AccessOptions, GetParentsResponse, Tenant, TenantFilter, TenantResolverError};

/// Public API trait for the `tenant_resolver` gateway (Version 1).
///
/// This trait is registered in `ClientHub`:
/// ```ignore
/// let resolver = hub.get::<dyn TenantResolverClientV1>()?;
/// ```
///
/// Other modules use this trait to interact with the tenant resolver gateway.
/// The gateway implementation selects the appropriate plugin and delegates the call.
#[async_trait]
pub trait TenantResolverClientV1: Send + Sync {
    /// Resolves and returns the root tenant.
    async fn get_root_tenant(&self, ctx: &SecurityContext) -> Result<Tenant, TenantResolverError>;

    /// Lists tenants with cursor-based pagination.
    ///
    /// Only `$select`, `limit`, and `cursor` are guaranteed to be supported by this example implementation.
    async fn list_tenants(
        &self,
        ctx: &SecurityContext,
        filter: TenantFilter,
        query: ODataQuery,
    ) -> Result<Page<Tenant>, TenantResolverError>;

    /// Returns all parents (direct and indirect) of the given tenant.
    ///
    /// Target tenant is included in the response. Parents are ordered from
    /// the direct parent to the top-level parent (i.e., root).
    async fn get_parents(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
    ) -> Result<GetParentsResponse, TenantResolverError>;

    /// Returns all children (direct and indirect) of the given tenant.
    ///
    /// Children are returned in pre-order traversal (parent before its subtree).
    /// `max_depth` controls depth: 0 = unlimited, 1 = direct children only, etc.
    async fn get_children(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
        max_depth: u32,
    ) -> Result<Vec<Tenant>, TenantResolverError>;
}

/// Internal plugin API trait (Version 1).
///
/// Each plugin registers this trait with a scoped `ClientHub` entry
/// using its GTS instance ID as the scope.
///
/// Plugins implement this trait to provide tenant resolution functionality.
/// The gateway calls this on the selected plugin implementation.
#[async_trait]
pub trait TenantResolverPluginClientV1: Send + Sync {
    /// Returns the root tenant as resolved by this plugin.
    async fn get_root_tenant(&self, ctx: &SecurityContext) -> Result<Tenant, TenantResolverError>;

    /// Lists tenants with cursor-based pagination.
    async fn list_tenants(
        &self,
        ctx: &SecurityContext,
        filter: TenantFilter,
        query: ODataQuery,
    ) -> Result<Page<Tenant>, TenantResolverError>;

    /// Returns all parents (direct and indirect) of the given tenant.
    ///
    /// Target tenant is included in the response. Parents are ordered from
    /// the direct parent to the top-level parent (i.e., root).
    async fn get_parents(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
    ) -> Result<GetParentsResponse, TenantResolverError>;

    /// Returns all children (direct and indirect) of the given tenant.
    ///
    /// Children are returned in pre-order traversal (parent before its subtree).
    /// `max_depth` controls depth: 0 = unlimited, 1 = direct children only, etc.
    async fn get_children(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
        max_depth: u32,
    ) -> Result<Vec<Tenant>, TenantResolverError>;
}
