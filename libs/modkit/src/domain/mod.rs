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
//! - Implement `DomainSafe` and `DomainModel` for the type
//! - Verify at compile-time that all fields are also `DomainSafe`
//!
//! # Enforcement
//!
//! Domain services can use trait bounds to ensure they only work with domain-safe types:
//!
//! ```rust,ignore
//! pub trait UserRepository: Send + Sync {
//!     type Model: DomainModel;
//!     
//!     async fn find(&self, id: Uuid) -> Result<Option<Self::Model>>;
//! }
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use uuid::Uuid;

/// Marker trait for types that are safe to use in the domain layer.
///
/// Types implementing this trait are guaranteed to be free of infrastructure
/// dependencies such as:
/// - Database types (`sqlx`, `sea_orm`)
/// - HTTP types (http, axum, hyper)
/// - Framework-specific types
///
/// # Implementing
///
/// Do NOT implement this trait manually. Use the `#[domain_model]` attribute macro
/// which performs compile-time validation that all fields are also `DomainSafe`.
///
/// # Blanket Implementations
///
/// Common standard library and third-party types have blanket implementations:
/// - Primitives: `bool`, `i8`..`i128`, `u8`..`u128`, `f32`, `f64`, `char`, `String`, `&str`
/// - Collections: `Vec<T>`, `Option<T>`, `Result<T, E>`, `HashMap<K, V>`, etc.
/// - Common crates: `uuid::Uuid`, `chrono::DateTime`, `rust_decimal::Decimal`
pub trait DomainSafe {}

/// Marker trait for domain models (business entities).
///
/// Domain models represent core business concepts and should:
/// - Contain only business-relevant data
/// - Be independent of persistence mechanisms
/// - Be serializable for transfer between layers
///
/// # Usage
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
pub trait DomainModel: DomainSafe + Send + Sync {}

/// Marker trait for domain errors.
///
/// Domain errors represent business rule violations and should not
/// contain infrastructure-specific error types.
pub trait DomainErrorMarker: DomainSafe + std::error::Error + Send + Sync {}

// =============================================================================
// Blanket implementations for primitives
// =============================================================================

impl DomainSafe for () {}
impl DomainSafe for bool {}
impl DomainSafe for char {}
impl DomainSafe for str {}
impl DomainSafe for String {}

impl DomainSafe for i8 {}
impl DomainSafe for i16 {}
impl DomainSafe for i32 {}
impl DomainSafe for i64 {}
impl DomainSafe for i128 {}
impl DomainSafe for isize {}

impl DomainSafe for u8 {}
impl DomainSafe for u16 {}
impl DomainSafe for u32 {}
impl DomainSafe for u64 {}
impl DomainSafe for u128 {}
impl DomainSafe for usize {}

impl DomainSafe for f32 {}
impl DomainSafe for f64 {}

// =============================================================================
// Blanket implementations for common wrappers
// =============================================================================

impl<T: DomainSafe> DomainSafe for Option<T> {}
impl<T: DomainSafe> DomainSafe for Box<T> {}
impl<T: DomainSafe> DomainSafe for std::sync::Arc<T> {}
impl<T: DomainSafe> DomainSafe for std::rc::Rc<T> {}
impl<T: DomainSafe, E: DomainSafe> DomainSafe for Result<T, E> {}

// =============================================================================
// Blanket implementations for collections
// =============================================================================

impl<T: DomainSafe> DomainSafe for Vec<T> {}
impl<T: DomainSafe> DomainSafe for [T] {}
impl<T: DomainSafe, const N: usize> DomainSafe for [T; N] {}

impl<K: DomainSafe, V: DomainSafe, S: std::hash::BuildHasher> DomainSafe for HashMap<K, V, S> {}
impl<K: DomainSafe, V: DomainSafe> DomainSafe for BTreeMap<K, V> {}
impl<T: DomainSafe, S: std::hash::BuildHasher> DomainSafe for HashSet<T, S> {}
impl<T: DomainSafe> DomainSafe for BTreeSet<T> {}

// =============================================================================
// Blanket implementations for tuples
// =============================================================================

impl<A: DomainSafe> DomainSafe for (A,) {}
impl<A: DomainSafe, B: DomainSafe> DomainSafe for (A, B) {}
impl<A: DomainSafe, B: DomainSafe, C: DomainSafe> DomainSafe for (A, B, C) {}
impl<A: DomainSafe, B: DomainSafe, C: DomainSafe, D: DomainSafe> DomainSafe for (A, B, C, D) {}

// =============================================================================
// Blanket implementations for common third-party crates
// =============================================================================

// uuid
impl DomainSafe for Uuid {}

// serde_json::Value (for flexible JSON data)
impl DomainSafe for serde_json::Value {}

// =============================================================================
// Implementations for modkit types that are domain-safe
// =============================================================================

// Page and PageInfo are domain-safe as they're just pagination wrappers
impl<T: DomainSafe> DomainSafe for modkit_odata::Page<T> {}
impl DomainSafe for modkit_odata::PageInfo {}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time test: these should compile
    #[allow(dead_code)]
    fn assert_domain_safe<T: DomainSafe>() {}

    #[test]
    fn test_primitives_are_domain_safe() {
        assert_domain_safe::<bool>();
        assert_domain_safe::<i32>();
        assert_domain_safe::<String>();
        assert_domain_safe::<f64>();
    }

    #[test]
    fn test_collections_are_domain_safe() {
        assert_domain_safe::<Vec<String>>();
        assert_domain_safe::<Option<i32>>();
        assert_domain_safe::<HashMap<String, i32>>();
    }

    #[test]
    fn test_third_party_types_are_domain_safe() {
        assert_domain_safe::<Uuid>();
        assert_domain_safe::<serde_json::Value>();
    }

    #[test]
    fn test_nested_types_are_domain_safe() {
        assert_domain_safe::<Vec<Option<Uuid>>>();
        assert_domain_safe::<HashMap<String, Vec<i32>>>();
        assert_domain_safe::<Result<String, String>>();
    }
}
