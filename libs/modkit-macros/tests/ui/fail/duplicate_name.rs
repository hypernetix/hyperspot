use modkit_macros::module;

#[module(name="a", name="b", capabilities=[stateful])]
pub struct X;

fn main() {}
