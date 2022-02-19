use anyhow::Result;
use examples_lib::data_discarder_service::DataDiscarder;
use examples_lib::data_discarder_types::DataDiscarderVariant;

use leaning_tower::{allocator::AllocatorService, mux_server::MuxServer};
use tracing::{error, info, Level};

async fn use_forever() -> Result<()> {
    let mut services = vec![];

    // All in all, the allocator will be serving 60 different data discarders.
    for _ in 0..10 {
        services.push(DataDiscarder::new(DataDiscarderVariant::Fast))
    }
    for _ in 0..20 {
        services.push(DataDiscarder::new(DataDiscarderVariant::Medium))
    }
    for _ in 0..30 {
        services.push(DataDiscarder::new(DataDiscarderVariant::Slow))
    }

    let service = AllocatorService::new(services);

    let handle = MuxServer::run("0.0.0.0:1234", service).await?;
    info!("DataDiscarder services now being allocated on demand");

    let _ = handle.await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::WARN)
        .init();
    info!("Running server");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("ctrl-c, stopping")
        }
        val = use_forever() => {
            error!("Robustness testing stopped: {:?}", val)
        }
    };

    Ok(())
}
