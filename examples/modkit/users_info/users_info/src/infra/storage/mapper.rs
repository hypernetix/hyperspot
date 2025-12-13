use crate::infra::storage::entity::Model as UserEntity;
use user_info_sdk::User;

/// Convert a database entity to a contract model (owned version)
impl From<UserEntity> for User {
    fn from(e: UserEntity) -> Self {
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
impl From<&UserEntity> for User {
    fn from(e: &UserEntity) -> Self {
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
