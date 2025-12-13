//! Domain service for calculator
//!
//! Contains the core business logic for accumulator operations.

use tracing::debug;

/// Domain service that performs accumulator operations.
///
/// This is a simple stateless service that implements the core
/// addition logic. It's registered in ClientHub and used by
/// the gRPC server.
#[derive(Clone, Default)]
pub struct Service;

impl Service {
    /// Create a new service.
    pub fn new() -> Self {
        Self
    }

    /// Add two numbers and return the sum.
    pub fn add(&self, a: i64, b: i64) -> i64 {
        debug!(a, b, "performing addition");
        a + b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let service = Service::new();
        assert_eq!(service.add(10, 20), 30);
    }

    #[test]
    fn test_add_negative() {
        let service = Service::new();
        assert_eq!(service.add(-5, 3), -2);
    }
}
