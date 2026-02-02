use modkit_macros::module;

#[module(name="x", deps="db")]
pub struct X;

fn main() {}
