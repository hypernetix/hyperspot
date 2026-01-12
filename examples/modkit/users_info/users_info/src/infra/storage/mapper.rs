use crate::infra::storage::entity;
use user_info_sdk::{Address, City, User};

/// Convert a database entity to a contract model (owned version)
impl From<entity::user::Model> for User {
    fn from(e: entity::user::Model) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            email: e.email,
            display_name: e.display_name,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// Convert a database entity to a contract model (by-ref version)
impl From<&entity::user::Model> for User {
    fn from(e: &entity::user::Model) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            email: e.email.clone(),
            display_name: e.display_name.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// Convert a city database entity to a contract model (owned version)
impl From<entity::city::Model> for City {
    fn from(e: entity::city::Model) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            name: e.name,
            country: e.country,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// Convert a city database entity to a contract model (by-ref version)
impl From<&entity::city::Model> for City {
    fn from(e: &entity::city::Model) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            name: e.name.clone(),
            country: e.country.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// Convert an address database entity to a contract model (owned version)
impl From<entity::address::Model> for Address {
    fn from(e: entity::address::Model) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            user_id: e.user_id,
            city_id: e.city_id,
            street: e.street,
            postal_code: e.postal_code,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// Convert an address database entity to a contract model (by-ref version)
impl From<&entity::address::Model> for Address {
    fn from(e: &entity::address::Model) -> Self {
        Self {
            id: e.id,
            tenant_id: e.tenant_id,
            user_id: e.user_id,
            city_id: e.city_id,
            street: e.street.clone(),
            postal_code: e.postal_code.clone(),
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}
