use modkit_macros::lifecycle;
use tokio_util::sync::CancellationToken;

struct Y;

#[lifecycle(method="run", await_ready=true)]
impl Y {
    async fn run(&self, _cancel: CancellationToken, _not_ready: u32) {}
}

fn main() {}

