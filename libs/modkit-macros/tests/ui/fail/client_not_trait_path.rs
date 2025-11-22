use modkit_macros::module;

#[module(name="x", capabilities=[stateful], client=123)]
pub struct X;

fn main() {}
