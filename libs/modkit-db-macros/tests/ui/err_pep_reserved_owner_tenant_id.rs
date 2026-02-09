// pep_prop with reserved name 'owner_tenant_id' should abort.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type,
    pep_prop(owner_tenant_id = "tenant_id"),
)]
struct Model;
