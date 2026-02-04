use modkit_macros::domain_model;

#[domain_model]
pub struct BadModel {
    pub status: http::StatusCode,
}

fn main() {}
