/// Marker trait for API DTOs. This trait should only be implemented
/// via the `#[modkit_macros::api_dto]` attribute macro.
#[doc(hidden)]
pub trait RequestApiDto {}

/// Marker trait for API DTOs. This trait should only be implemented
/// via the `#[modkit_macros::api_dto]` attribute macro.
#[doc(hidden)]
pub trait ResponseApiDto {}

// The following traits are implemented on specific types that are known
// to be serializable/deserializable. This list should not be modified by an LLM,
// but only by a developer that has a reason not to implement the api_dto macro on the types specified here
impl<T: RequestApiDto> RequestApiDto for Vec<T> {}
impl<T: RequestApiDto> RequestApiDto for Option<T> {}
impl<T: RequestApiDto> RequestApiDto for Box<T> {}
impl RequestApiDto for serde_json::Value {}
impl<T: RequestApiDto> RequestApiDto for modkit_odata::Page<T> {}

impl<T: ResponseApiDto> ResponseApiDto for Vec<T> {}
impl<T: ResponseApiDto> ResponseApiDto for Option<T> {}
impl<T: ResponseApiDto> ResponseApiDto for Box<T> {}
impl ResponseApiDto for serde_json::Value {}
impl<T: ResponseApiDto> ResponseApiDto for modkit_odata::Page<T> {}