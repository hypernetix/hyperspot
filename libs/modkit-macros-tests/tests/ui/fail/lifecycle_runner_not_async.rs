use modkit_macros::lifecycle;
use tokio_util::sync::CancellationToken;

struct Y;

#[lifecycle(method="run")]
impl Y {
    fn run(&self, _cancel: CancellationToken) {}
}

fn main() {}
