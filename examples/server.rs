use anyhow::Result;
use leaning_tower::{server_runner, shared};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    server_runner::run(shared::SERVER_BIND_ADDR).await
}
