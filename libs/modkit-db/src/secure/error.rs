use uuid::Uuid;

/// Errors that can occur during scoped query execution.
#[derive(thiserror::Error, Debug)]
pub enum ScopeError {
    /// Database error occurred during query execution.
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// Invalid scope configuration.
    #[error("invalid scope: {0}")]
    Invalid(&'static str),

    /// Tenant isolation violation: `tenant_id` is not included in the current scope.
    #[error("access denied: tenant_id not present in security scope ({tenant_id})")]
    TenantNotInScope { tenant_id: Uuid },

    /// Operation denied - entity not accessible in current security scope.
    #[error("access denied: {0}")]
    Denied(&'static str),
}
