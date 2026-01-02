// This test demonstrates the actual macro expansion
// Run with: cargo expand --package modkit-sdk-macros --test expansion_demo

use modkit_sdk_macros::ODataSchema;

#[derive(ODataSchema)]
#[allow(dead_code)]
struct DemoDto {
    id: uuid::Uuid,
    email: String,
    #[odata(name = "created_at")]
    created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use modkit_odata::SortDir;
    use modkit_sdk::odata::{AsFieldKey, FilterExpr, QueryBuilder, Schema};

    #[test]
    fn demo_field_enum() {
        let _ = DemoDtoField::Id;
        let _ = DemoDtoField::Email;
        let _ = DemoDtoField::CreatedAt;
    }

    #[test]
    fn demo_schema_mapping() {
        assert_eq!(DemoDtoSchema::field_name(DemoDtoField::Id), "id");
        assert_eq!(DemoDtoSchema::field_name(DemoDtoField::Email), "email");
        assert_eq!(
            DemoDtoSchema::field_name(DemoDtoField::CreatedAt),
            "created_at"
        );
    }

    #[test]
    fn demo_typed_constructors() {
        let id = demo_dto::id();
        let email = demo_dto::email();
        let created_at = demo_dto::created_at();

        assert_eq!(id.name(), "id");
        assert_eq!(email.name(), "email");
        assert_eq!(created_at.name(), "created_at");
    }

    #[test]
    fn demo_query_building() {
        let dto_id = uuid::Uuid::new_v4();

        let id = demo_dto::id();
        let email = demo_dto::email();

        let query = QueryBuilder::<DemoDtoSchema>::new()
            .filter(
                demo_dto::id()
                    .eq(dto_id)
                    .and(demo_dto::email().contains("@example.com")),
            )
            .order_by(demo_dto::created_at(), SortDir::Desc)
            .select(&[
                &id as &dyn AsFieldKey<DemoDtoSchema>,
                &email as &dyn AsFieldKey<DemoDtoSchema>,
            ])
            .page_size(25)
            .build();

        assert!(query.has_filter());
        assert_eq!(query.order.0.len(), 1);
        assert_eq!(query.limit, Some(25));
    }
}
