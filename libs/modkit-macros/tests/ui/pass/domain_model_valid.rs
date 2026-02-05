use modkit::domain::DomainModel;
use modkit_macros::domain_model;

#[domain_model]
pub struct User {
    pub id: i64,
    pub name: String,
    pub active: bool,
}

#[domain_model]
pub enum Status {
    Active,
    Inactive,
}

// Verify that DomainModel trait is implemented
fn assert_domain_model<T: DomainModel>(_: &T) {}

fn main() {
    let user = User { id: 1, name: String::from("test"), active: true };
    let status = Status::Active;

    assert_domain_model(&user);
    assert_domain_model(&status);
}
