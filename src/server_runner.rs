use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpListener;
use tower::{buffer::Buffer, Service};
use tracing::{error, info, info_span, Instrument};

use crate::{allocator, resource_filter::Describable, server_peer_handler};

pub async fn run<S, R, HR>(
    bind_address: &str,
    services: Vec<S>,
) -> std::result::Result<(), String>
where
    S: Service<R> + Send + 'static,
    S: Describable<HR>,
    R: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync + Into<tower::BoxError>,
    HR: Send + DeserializeOwned + Serialize + std::fmt::Debug + 'static + Clone,
{
    info!("Initializing server running at `{bind_address}`");

    // let tcp = TcpListener::bind(bind_address).await?;
    let tcp = TcpListener::bind(bind_address)
        .await
        .map_err(|e| e.to_string())?;

    let allocator = Buffer::new(allocator::AllocatorService::new(services), 10);

    let mut clients = 0;
    loop {
        let (stream, peer) = match tcp.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Cannot accept: {:?}", e);
                return Ok(());
            }
        };

        let allocator = allocator.clone();
        clients += 1;

        tokio::spawn(
            // server_peer_handler::handle(stream, allocator).instrument(info_span!(
            //     "peer-handler",
            //     clients,
            //     ?peer
            // )),
            server_peer_handler::handle(stream, allocator),
        );
    }
}
