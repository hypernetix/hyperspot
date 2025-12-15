use uuid::Uuid;

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

    /// Returns the special root/system subject.
    #[must_use]
    pub fn root() -> Self {
        Self::new(crate::constants::ROOT_SUBJECT_ID)
    }

    /// Returns true if this subject represents the root/system identity.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.id == crate::constants::ROOT_SUBJECT_ID
    }
}
