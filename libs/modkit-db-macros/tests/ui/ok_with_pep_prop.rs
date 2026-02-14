// Entity with pep_prop alongside standard dimension columns - macro should expand.
// Note: This test only validates macro expansion, not the full trait implementation.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type,
    pep_prop(department_id = "department_id"),
    pep_prop(region_id = "region_id"),
)]
struct Model {
    tenant_id: String,
    id: String,
    department_id: String,
    region_id: String,
}

fn main() {}
