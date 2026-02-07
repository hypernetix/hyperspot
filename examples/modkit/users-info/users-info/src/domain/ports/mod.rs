pub mod audit;

pub use audit::AuditPort;

/// Output port: publish domain events (no knowledge of transport).
pub trait EventPublisher<E>: Send + Sync + 'static {
    fn publish(&self, event: &E);
}
