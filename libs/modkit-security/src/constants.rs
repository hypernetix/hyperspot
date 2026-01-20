use uuid::Uuid;
use uuid::uuid;

/// Default tenant ID for single-tenant or auth-disabled deployments.
///
/// Used when:
/// - Auth is disabled (on-premises single-user mode)
/// - Default/fallback tenant ID is needed (e.g., migrations, examples)
///
/// In multi-tenant production deployments, tenant IDs come from
/// the authentication layer or tenant resolver.
pub const DEFAULT_TENANT_ID: Uuid = uuid!("00000000-df51-5b42-9538-d2b56b7ee953");

/// Default subject ID for single-tenant or auth-disabled deployments.
///
/// Used when:
/// - Auth is disabled (on-premises single-user mode)
/// - Default/fallback subject ID is needed
///
/// In production deployments, subject IDs come from the authentication layer.
pub const DEFAULT_SUBJECT_ID: Uuid = uuid!("11111111-6a88-4768-9dfc-6bcd5187d9ed");

/// Default GTS type ID placeholder.
pub const GTS_DEFAULT_TYPE_ID: Uuid = uuid!("22222222-0000-0000-0000-000000000001");
