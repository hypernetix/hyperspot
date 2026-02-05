use modkit_macros::domain_model;

// Mock infra module to ensure the type resolves
mod infra {
    pub struct UserRepository;
}

// `infra::` paths are allowed in domain models.
// Architectural enforcement (preventing infra in domain layer) is handled
// by dylint rules, not by the macro itself.
#[domain_model]
pub struct GoodModel {
    pub repo: infra::UserRepository,
}

fn main() {}
