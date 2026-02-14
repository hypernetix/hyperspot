// pep_prop with reserved name 'id' should abort.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type,
    pep_prop(id = "id"),
)]
struct Model;
