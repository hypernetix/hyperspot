/// Security requirement - defines required resource and action
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecRequirement {
    pub resource: &'static str,
    pub action: &'static str,
}

impl SecRequirement {
    pub fn new(resource: &'static str, action: &'static str) -> Self {
        Self { resource, action }
    }
}
