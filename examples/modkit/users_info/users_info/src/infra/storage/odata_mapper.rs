//! Infrastructure layer mapping from type-safe `FilterNode` to `SeaORM` Conditions.
//!
//! This module is the ONLY place where we map from DTO-level filter fields to
//! `SeaORM` Column types. The API and domain layers work exclusively with `FilterNode`
//! and do not know about `SeaORM` Column enums.
//!
//! This module provides the complete `OData` mapping including filtering, ordering,
//! and cursor extraction - all using the type-safe `FilterField` approach.

use modkit_db::odata::filter::FilterNode;
use modkit_db::odata::sea_orm_filter::{
    filter_node_to_condition, FieldToColumn, ODataFieldMapping,
};
use sea_orm::Condition;

use crate::api::rest::dto::UserDtoFilterField;
use crate::infra::storage::entity::{Column, Entity, Model};

/// Complete `OData` mapper for `users_info`.
///
/// This is the only users_info-specific code needed for `OData` operations.
/// It maps `UserDtoFilterField` to database columns and extracts cursor values.
pub struct UserODataMapper;

impl FieldToColumn<UserDtoFilterField> for UserODataMapper {
    type Column = Column;

    fn map_field(field: UserDtoFilterField) -> Column {
        match field {
            UserDtoFilterField::Id => Column::Id,
            UserDtoFilterField::Email => Column::Email,
            UserDtoFilterField::CreatedAt => Column::CreatedAt,
        }
    }
}

impl ODataFieldMapping<UserDtoFilterField> for UserODataMapper {
    type Entity = Entity;

    fn extract_cursor_value(model: &Model, field: UserDtoFilterField) -> sea_orm::Value {
        match field {
            UserDtoFilterField::Id => sea_orm::Value::Uuid(Some(Box::new(model.id))),
            UserDtoFilterField::Email => {
                sea_orm::Value::String(Some(Box::new(model.email.clone())))
            }
            UserDtoFilterField::CreatedAt => {
                sea_orm::Value::ChronoDateTimeUtc(Some(Box::new(model.created_at)))
            }
        }
    }
}

/// Map a `FilterNode`<UserDtoFilterField> to a `SeaORM` Condition.
///
/// This function is provided for compatibility but is no longer needed
/// if you use `paginate_odata` directly, which handles filtering internally.
///
/// # Arguments
///
/// * `filter` - The type-safe filter node from the API/domain layer
///
/// # Returns
///
/// A `SeaORM` Condition that can be applied to a query
pub fn filter_to_condition(filter: &FilterNode<UserDtoFilterField>) -> Result<Condition, String> {
    filter_node_to_condition::<UserDtoFilterField, UserODataMapper>(filter)
}
