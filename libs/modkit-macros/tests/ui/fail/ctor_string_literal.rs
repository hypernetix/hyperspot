use modkit_macros::module;

#[module(name="x", capabilities=[stateful], ctor="X::new()")]
pub struct X;

fn main() {}
