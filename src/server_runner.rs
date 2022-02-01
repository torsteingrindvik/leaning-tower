use anyhow::Result;
use tokio::net::TcpListener;
use tower::buffer::Buffer;
use tracing::{error, info, info_span, Instrument};

use crate::{map_allocator, server_peer_handler};

pub async fn run(bind_address: &str) -> Result<()> {
    info!("Initializing server running at `{bind_address}`");

    let tcp = TcpListener::bind(bind_address).await?;

    let allocator = Buffer::new(
        map_allocator::MapAllocatorService::new(
            ["Ding".into(), "Bong".into(), "Gang".into()].into(),
        ),
        10,
    );

    let mut clients = 0;
    loop {
        let (stream, peer) = match tcp.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Cannot accept: {:?}", e);
                return Ok(());
            }
        };

        let peer_service = allocator.clone();
        clients += 1;

        tokio::spawn(
            server_peer_handler::handle(stream, peer_service).instrument(info_span!(
                "peer-handler",
                clients,
                ?peer
            )),
        );
    }
}
