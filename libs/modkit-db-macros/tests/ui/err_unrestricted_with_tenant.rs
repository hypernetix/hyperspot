// Conflicting flags: 'unrestricted' cannot be used together with 'tenant_col'.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(unrestricted, tenant_col = "tenant_id")]
struct Model;


