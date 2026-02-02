use modkit_macros::lifecycle;

struct Y;

#[lifecycle(method="run")] // no such method in impl below
impl Y {
    // nothing
}

fn main() {}
