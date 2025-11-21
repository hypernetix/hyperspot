use modkit_macros::generate_clients;

#[generate_clients]
#[async_trait::async_trait]
pub trait BadApi: Send + Sync {
    async fn get_item(&self, req: String) -> Result<String, anyhow::Error>;
}

fn main() {}

