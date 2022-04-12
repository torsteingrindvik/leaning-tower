use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{future::select_ok, stream, Future, StreamExt};
use leaning_tower::{
    allocator::AllocatorService, allocator_client::AllocatorClientService, mux_server,
    resource_filter::Describable,
};
use rand::Rng;
use tokio::sync::Notify;
use tower::{BoxError, Service, ServiceExt};
use tracing::info;

////////////////////////////////////////////////////////////////////////////////
// Settings
////////////////////////////////////////////////////////////////////////////////
const SERVER_ADDR: &str = "0.0.0.0:5566";
const SERVER_ADDR2: &str = "0.0.0.0:5567";
const SERVICES: usize = 10;
const USERS: usize = 100;
const MODULO: usize = 3;

////////////////////////////////////////////////////////////////////////////////
// A simple describable service which returns requests (strings) in uppercase
////////////////////////////////////////////////////////////////////////////////
struct IndexedService(usize);

impl Service<String> for IndexedService {
    type Response = String;
    type Error = BoxError;
    #[allow(clippy::type_complexity)]
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: String) -> Self::Future {
        Box::pin(async move { Ok(req.to_ascii_uppercase()) })
    }
}

impl Describable<usize> for IndexedService {
    fn describe(&self) -> usize {
        self.0
    }
}

////////////////////////////////////////////////////////////////////////////////
// Run the server side: Waits for requests to get hold of some described
// service.
// Creates 10 services which may be allocated.
////////////////////////////////////////////////////////////////////////////////
async fn run(addr: &str, notify: Arc<Notify>) -> Result<(), BoxError> {
    let mut services = vec![];

    for index in 0..SERVICES {
        services.push(IndexedService(index % MODULO))
    }

    let service = AllocatorService::new(services);

    let handle = mux_server::run(addr, service).await?;
    info!(?addr, "Letting the service allocator run forever");

    notify.notify_one();
    let _ = handle.await?;
    info!("Done serving many resource");

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Start the work from the client.
// Starts `USERS` requests for resources.
//
// Each user will ask for a resource indexed by the user's index,
// modulo `MODULO`.
//
// When a resource is allocated, the user will do a simple request on it
// and log the response.
////////////////////////////////////////////////////////////////////////////////
async fn work() -> Result<(), BoxError> {
    let allocator: AllocatorClientService<_, IndexedService, _> =
        AllocatorClientService::new(SERVER_ADDR).await?;

    let _result = stream::iter(0usize..USERS)
        .map(|index| (index, allocator.clone()))
        .for_each_concurrent(None, |(index, mut allocator)| async move {
            let index_mod = index % MODULO;
            let mut svc = allocator
                .ready()
                .await
                .unwrap()
                .call(index_mod)
                .await
                .unwrap()
                .unwrap();

            let response = svc
                .ready()
                .await
                .unwrap()
                .call(format!("Hi from {}", index))
                .await
                .unwrap();
            info!(%index, %index_mod, "Response: {response}");
        })
        .await;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Simulates a more intricate setup where there might be several
// allocators stored behind a map.
////////////////////////////////////////////////////////////////////////////////
async fn work_intricate() -> Result<(), BoxError> {
    let allocator: AllocatorClientService<_, IndexedService, _> =
        AllocatorClientService::new_labelled(SERVER_ADDR, "server-1").await?;
    let allocator2 = AllocatorClientService::new_labelled(SERVER_ADDR2, "server-2").await?;

    let allocator_map = HashMap::from([(SERVER_ADDR, allocator), (SERVER_ADDR2, allocator2)]);

    let _result = stream::iter(0usize..USERS)
        .map(|index| (index, allocator_map.clone()))
        .for_each_concurrent(None, |(index, allocator_map)| async move {
            let index_mod = index % MODULO;

            let ((svc, addr), _) = select_ok(allocator_map.iter().map(|(addr, allocator)| {
                Box::pin(async move {
                    let svc = allocator
                        .clone()
                        .ready()
                        .await
                        .unwrap()
                        .call(index_mod)
                        .await
                        .unwrap();

                    Result::<_, BoxError>::Ok((svc, addr))
                })
            }))
            .await
            .unwrap();

            let response = svc
                .unwrap()
                .ready()
                .await
                .unwrap()
                .call(format!("Hi from {} (behind {})", index, addr))
                .await
                .unwrap();
            info!(%index, %index_mod, "Response: {response}");

            tokio::time::sleep(Duration::from_millis(
                rand::thread_rng().gen_range(10..=2000),
            ))
            .await;
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_use() {
    tracing_subscriber::fmt::init();

    // Init servers
    let notify = Arc::new(Notify::new());
    tokio::spawn(run(SERVER_ADDR, notify.clone()));
    tokio::spawn(run(SERVER_ADDR2, notify.clone()));

    // Wait until servers are ready
    notify.notified().await;
    notify.notified().await;

    // Do all the work.
    let _ = work().await;
    let _ = work_intricate().await;
}
