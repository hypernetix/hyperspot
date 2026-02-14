// Duplicate pep_prop property names should abort.

use modkit_db_macros::Scopable;

#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type,
    pep_prop(department_id = "department_id"),
    pep_prop(department_id = "dept_id"),
)]
struct Model;
