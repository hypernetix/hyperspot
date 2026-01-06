use modkit_odata_macros::ODataFilterable;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(ODataFilterable)]
pub struct UserQuery {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,

    #[odata(filter(kind = "String"))]
    pub email: String,

    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: OffsetDateTime,
}

pub use UserQueryFilterField as UserFilterField;

#[derive(ODataFilterable)]
pub struct CityQuery {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,

    #[odata(filter(kind = "String"))]
    pub name: String,

    #[odata(filter(kind = "String"))]
    pub country: String,

    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: OffsetDateTime,
}

pub use CityQueryFilterField as CityFilterField;

#[derive(ODataFilterable)]
pub struct LanguageQuery {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,

    #[odata(filter(kind = "String"))]
    pub code: String,

    #[odata(filter(kind = "String"))]
    pub name: String,

    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: OffsetDateTime,
}

pub use LanguageQueryFilterField as LanguageFilterField;
