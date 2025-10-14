// Duplicate attribute: tenant_col specified twice should abort.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(tenant_col = "tenant_id")]
#[secure(tenant_col = "tenant2_id")]
struct Model;


