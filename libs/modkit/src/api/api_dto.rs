/// Marker trait for API DTOs. This trait should only be implemented
/// via the `#[modkit_macros::api_dto]` attribute macro.
#[doc(hidden)]
pub trait RequestApiDto {}

/// Marker trait for API DTOs. This trait should only be implemented
/// via the `#[modkit_macros::api_dto]` attribute macro.
#[doc(hidden)]
pub trait ResponseApiDto {}

macro_rules! impl_api_dto_trait {
    (generic: $($t:ty),+ $(,)?) => {
        $(
            impl<T: RequestApiDto> RequestApiDto for $t {}
            impl<T: ResponseApiDto> ResponseApiDto for $t {}
        )*
    };
    (concrete: $($t:ty),+ $(,)?) => {
        $(
            impl RequestApiDto for $t {}
            impl ResponseApiDto for $t {}
        )*
    };
}

// The following traits are implemented on specific types that are known
// to be serializable/deserializable. This list should not be modified by an LLM,
// but only by a developer that has a reason not to implement the api_dto macro on the types specified here
impl_api_dto_trait!(
    generic:
    Vec<T>,
    Option<T>,
    Box<T>,
    modkit_odata::Page<T>,
    Result<T, anyhow::Error>,
    Result<T, modkit_errors::Problem>,
);
impl_api_dto_trait!(
    concrete:
    serde_json::Value,
    modkit_errors::Problem,
);
