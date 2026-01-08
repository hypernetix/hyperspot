use uuid::Uuid;

/// Represents the actor (user or service) making a request.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Subject {
    pub(crate) id: Uuid,
    // future: kind (user/service), realm, display, etc.
}

impl Subject {
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    #[inline]
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
