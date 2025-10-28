use uuid::Uuid;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subject {
    pub(crate) id: Uuid,
    // future: kind (user/service), realm, display, etc.
}

impl Subject {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
    #[inline]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the special root/system subject.
    pub fn root() -> Self {
        Self::new(crate::constants::ROOT_SUBJECT_ID)
    }

    /// Returns true if this subject represents the root/system identity.
    pub fn is_root(&self) -> bool {
        self.id == crate::constants::ROOT_SUBJECT_ID
    }
}
