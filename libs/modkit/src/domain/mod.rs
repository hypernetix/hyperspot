//! Domain Layer Marker Traits
//!
//! This module provides marker traits for enforcing domain-driven design boundaries
//! at compile time. Types marked with these traits are guaranteed to be free of
//! infrastructure dependencies (`sqlx`, `sea_orm`, `http`, `axum`, etc.).
//!
//! # Usage
//!
//! Use the `#[domain_model]` attribute macro to mark domain types:
//!
//! ```rust,ignore
//! use modkit_macros::domain_model;
//!
//! #[domain_model]
//! pub struct User {
//!     pub id: Uuid,
//!     pub email: String,
//!     pub created_at: DateTime<Utc>,
//! }
//! ```
//!
//! The macro will:
//! - Implement `DomainModel` for the type
//! - Validate at macro-expansion time that all fields are free of infrastructure types
//! - Generate clear error messages if forbidden types are detected
//!
//! # Enforcement
//!
//! Domain services can use trait bounds to ensure they only work with domain types:
//!
//! ```rust,ignore
//! pub trait UserRepository: Send + Sync {
//!     type Model: DomainModel;
//!
//!     async fn find(&self, id: Uuid) -> Result<Option<Self::Model>>;
//! }
//! ```
//!
//! # Validation Strategy
//!
//! The `#[domain_model]` macro performs validation by checking field type names against
//! a list of forbidden patterns (e.g., `sqlx::`, `http::`, `sea_orm::`). This provides
//! clear error messages at macro expansion time, similar to how `#[api_dto]` validates
//! its arguments.
//!
//! Additional enforcement is provided by Dylint lints:
//! - `DE0301`: Prohibits infrastructure imports in domain layer
//! - `DE0308`: Prohibits HTTP types in domain layer

/// Marker trait for domain models (business entities).
///
/// Domain models represent core business concepts and should:
/// - Contain only business-relevant data
/// - Be independent of persistence mechanisms
/// - Be free of infrastructure dependencies
///
/// # Usage
///
/// Use the `#[domain_model]` attribute macro to implement this trait:
///
/// ```rust,ignore
/// #[domain_model]
/// pub struct Order {
///     pub id: Uuid,
///     pub customer_id: Uuid,
///     pub total: Decimal,
///     pub status: OrderStatus,
/// }
/// ```
///
/// The macro validates field types at expansion time and generates clear error
/// messages if forbidden types (e.g., `sqlx::PgPool`, `http::StatusCode`) are detected.
///
/// # Thread Safety
///
/// This trait does not require `Send + Sync`. If you need thread-safe domain models,
/// wrap them in `Arc<T>` at the point of use.
pub trait DomainModel {}

/// Marker trait for domain errors.
///
/// Domain errors represent business rule violations and should not
/// contain infrastructure-specific error types.
pub trait DomainErrorMarker: std::error::Error + Send + Sync {}

#[cfg(test)]
mod tests {
    use super::*;

    // DomainModel is a simple marker trait - no trait bounds to test
    #[allow(dead_code)]
    fn assert_domain_model<T: DomainModel>() {}

    #[test]
    fn test_domain_model_trait_exists() {
        // Test that the trait is defined and can be used as a bound
        // Actual validation is done by the #[domain_model] macro
        fn _accepts_domain_model<T: DomainModel>(_: T) {}
    }
}
