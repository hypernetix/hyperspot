use modkit_macros::generate_clients;

#[generate_clients(grpc_client = "crate::Service")]
#[async_trait::async_trait]
pub trait BadApi: Send + Sync {
    async fn get_item(&mut self, req: String) -> Result<String, anyhow::Error>;
}

fn main() {}
