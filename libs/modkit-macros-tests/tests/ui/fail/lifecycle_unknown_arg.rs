use modkit_macros::module;

#[module(name="x", capabilities=[stateful], lifecycle(entry="serve", foo="bar"))]
pub struct X;

fn main() {}
