use std::sync::Arc;

use modkit_macros::domain_model;
use tracing::{debug, info, instrument};

use crate::domain::error::DomainError;
use crate::domain::repos::{AddressesRepository, UsersRepository};
use crate::domain::service::DbProvider;
use modkit_odata::{ODataQuery, Page};
use modkit_security::{PolicyEngineRef, SecurityContext};
use tenant_resolver_sdk::TenantResolverGatewayClient;
use time::OffsetDateTime;
use users_info_sdk::{Address, AddressPatch, NewAddress};
use uuid::Uuid;

#[domain_model]
pub struct AddressesService<R: AddressesRepository, U: UsersRepository> {
    db: Arc<DbProvider>,
    policy_engine: PolicyEngineRef,
    repo: Arc<R>,
    users_repo: Arc<U>,
    resolver: Arc<dyn TenantResolverGatewayClient>,
}

impl<R: AddressesRepository, U: UsersRepository> AddressesService<R, U> {
    pub fn new(
        db: Arc<DbProvider>,
        repo: Arc<R>,
        users_repo: Arc<U>,
        policy_engine: PolicyEngineRef,
        resolver: Arc<dyn TenantResolverGatewayClient>,
    ) -> Self {
        Self {
            db,
            policy_engine,
            repo,
            users_repo,
            resolver,
        }
    }
}

// Business logic methods
impl<R: AddressesRepository, U: UsersRepository> AddressesService<R, U> {
    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn get_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Address, DomainError> {
        debug!("Getting address by id");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get(&conn, &scope, id).await?;

        found.ok_or_else(|| DomainError::not_found("Address", id))
    }

    /// List addresses with cursor-based pagination
    #[instrument(skip(self, ctx, query))]
    pub async fn list_addresses_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<Address>, DomainError> {
        debug!("Listing addresses with cursor pagination");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let page = self.repo.list_page(&conn, &scope, query).await?;

        debug!("Successfully listed {} addresses in page", page.items.len());
        Ok(page)
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn get_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        debug!("Getting address by user_id");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get_by_user_id(&conn, &scope, user_id).await?;

        Ok(found)
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn get_address_by_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        self.get_user_address(ctx, user_id).await
    }

    #[allow(clippy::cognitive_complexity)]
    #[instrument(skip(self, ctx, address), fields(user_id = %user_id))]
    pub async fn put_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        address: NewAddress,
    ) -> Result<Address, DomainError> {
        info!("Upserting address for user");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let user = self
            .users_repo
            .get(&conn, &scope, user_id)
            .await?
            .ok_or_else(|| DomainError::user_not_found(user_id))?;

        let existing = self.repo.get_by_user_id(&conn, &scope, user_id).await?;

        let now = OffsetDateTime::now_utc();

        if let Some(existing_model) = existing {
            let mut updated: Address = existing_model;
            updated.city_id = address.city_id;
            updated.street = address.street;
            updated.postal_code = address.postal_code;
            updated.updated_at = now;

            let _ = self.repo.update(&conn, &scope, updated.clone()).await?;

            info!("Successfully updated address for user");
            Ok(updated)
        } else {
            let id = address.id.unwrap_or_else(Uuid::now_v7);

            let new_address = Address {
                id,
                tenant_id: user.tenant_id,
                user_id,
                city_id: address.city_id,
                street: address.street,
                postal_code: address.postal_code,
                created_at: now,
                updated_at: now,
            };

            let _ = self.repo.create(&conn, &scope, new_address.clone()).await?;

            info!("Successfully created address for user");
            Ok(new_address)
        }
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn delete_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        info!("Deleting address for user");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let rows_affected = self.repo.delete_by_user_id(&conn, &scope, user_id).await?;

        if rows_affected == 0 {
            return Err(DomainError::not_found("Address", user_id));
        }

        info!("Successfully deleted address for user");
        Ok(())
    }

    #[instrument(skip(self, ctx), fields(user_id = %new_address.user_id))]
    pub async fn create_address(
        &self,
        ctx: &SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, DomainError> {
        info!("Creating new address");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let now = OffsetDateTime::now_utc();
        let id = new_address.id.unwrap_or_else(Uuid::now_v7);

        let address = Address {
            id,
            tenant_id: new_address.tenant_id,
            user_id: new_address.user_id,
            city_id: new_address.city_id,
            street: new_address.street,
            postal_code: new_address.postal_code,
            created_at: now,
            updated_at: now,
        };

        let _ = self.repo.create(&conn, &scope, address.clone()).await?;

        info!("Successfully created address with id={}", address.id);
        Ok(address)
    }

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn update_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: AddressPatch,
    ) -> Result<Address, DomainError> {
        info!("Updating address");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get(&conn, &scope, id).await?;

        let mut current: Address = found.ok_or_else(|| DomainError::not_found("Address", id))?;

        if let Some(city_id) = patch.city_id {
            current.city_id = city_id;
        }
        if let Some(street) = patch.street {
            current.street = street;
        }
        if let Some(postal_code) = patch.postal_code {
            current.postal_code = postal_code;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let _ = self.repo.update(&conn, &scope, current.clone()).await?;

        info!("Successfully updated address");
        Ok(current)
    }

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn delete_address(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting address");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let deleted = self.repo.delete(&conn, &scope, id).await?;

        if !deleted {
            return Err(DomainError::not_found("Address", id));
        }

        info!("Successfully deleted address");
        Ok(())
    }
}
