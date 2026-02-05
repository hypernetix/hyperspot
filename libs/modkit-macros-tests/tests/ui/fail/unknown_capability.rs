use modkit_macros::module;

#[module(name="x", capabilities=[foo])]
pub struct X;

fn main() {}
