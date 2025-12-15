// Test DE0202: DTOs Not Referenced Outside API
// This lint checks for DTO type IMPORTS outside api layer

// Should trigger DE0202 - importing DTO in domain
use crate::api::rest::dto::UserDto;

#[derive(Debug, Clone)]
pub struct UserService;

impl UserService {
    pub fn process(&self) {
        // Using the import
        let _: Option<UserDto> = None;
    }
}
