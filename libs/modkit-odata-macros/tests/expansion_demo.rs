// This test demonstrates the actual macro expansion
// Run with: cargo expand --package modkit-odata-macros --test expansion_demo

use modkit_odata_macros::ODataSchema;

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
    use modkit_odata::schema::Schema;

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

    // Note: QueryBuilder integration test removed to avoid circular dependency.
    // Full integration tests with QueryBuilder belong in modkit-sdk tests.
}
