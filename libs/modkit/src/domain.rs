//! Domain layer support types.
//!
//! This module provides marker traits for domain-driven design (DDD) patterns.

/// Marker trait for domain model types.
///
/// Types marked with `#[domain_model]` attribute automatically implement this trait.
/// This enables compile-time verification that a type is a valid domain model,
/// free of infrastructure dependencies.
///
/// # Example
///
/// ```rust
/// use modkit_macros::domain_model;
/// use modkit::domain::DomainModel;
///
/// #[domain_model]
/// pub struct User {
///     pub id: i64,
///     pub name: String,
/// }
///
/// fn process<T: DomainModel>(_model: &T) {
///     // Only accepts valid domain models
/// }
///
/// let user = User { id: 1, name: String::from("test") };
/// process(&user);
/// ```
#[doc(hidden)]
pub trait DomainModel {}
