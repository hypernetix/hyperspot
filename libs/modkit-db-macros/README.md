# ModKit DB Macros

 Procedural macros used by `modkit-db`.

 Cargo package name: `cf-modkit-db-macros`
 Crate name (Rust import): `modkit_db_macros`

 ## Macros

 ### `#[derive(Scopable)]`

 Implements `modkit_db::secure::ScopableEntity` for a SeaORM entity.

 Put the derive on the `DeriveEntityModel` `Model` struct, together with a `#[secure(...)]` attribute.

 ```rust
 use modkit_db_macros::Scopable;
 use sea_orm::entity::prelude::*;
 use uuid::Uuid;

 #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
 #[sea_orm(table_name = "users")]
 #[secure(tenant_col = "tenant_id", resource_col = "id", no_owner, no_type)]
 pub struct Model {
     #[sea_orm(primary_key, auto_increment = false)]
     pub id: Uuid,
     pub tenant_id: Uuid,
     pub email: String,
 }
 ```

 #### `#[secure(...)]` rules

 Either:

 - **Global entity**
 
   - `#[secure(unrestricted)]`
   - Must not be combined with any other `secure` settings.

 Or: specify all scope dimensions explicitly (no defaults):

 - **Tenant**
 
   - `tenant_col = "..."` or `no_tenant`
 - **Resource**
 
   - `resource_col = "..."` or `no_resource`
 - **Owner**
 
   - `owner_col = "..."` or `no_owner`
 - **Type**
 
   - `type_col = "..."` or `no_type`

 `*_col` values are column names. The macro maps `snake_case` to the SeaORM column variant using `UpperCamelCase` (e.g. `tenant_id` -> `TenantId`).

 ## Notes

 OData derive macros (e.g. `ODataFilterable`) are not exported from this crate.

## License

Licensed under Apache-2.0.
