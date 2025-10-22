use uuid::Uuid;
use uuid::uuid;

/// Root (bootstrap) tenant and subject.
/// These are used only for system-level operations or bootstrap contexts.
pub const ROOT_TENANT_ID: Uuid = uuid!("00000000-df51-5b42-9538-d2b56b7ee953");
pub const ROOT_SUBJECT_ID: Uuid = uuid!("11111111-6a88-4768-9dfc-6bcd5187d9ed");
