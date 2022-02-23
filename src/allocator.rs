use std::{
    fmt::{Debug, Display},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::TryFutureExt;
use futures_core::Future;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{AcquireError, OwnedSemaphorePermit, Semaphore};
use tower::{buffer::Buffer, Service};
use tracing::{debug, error, info_span, Instrument};

use crate::{mux_server, resource_filter::Describable};

pub struct Resource<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync,
    D: PartialEq,
{
    inner: Buffer<S, Req>,
    semaphore: Arc<Semaphore>,
    description: D,
}

impl<S, Req, D> Debug for Resource<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync,
    D: PartialEq + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("semaphore", &self.semaphore)
            .field("description", &self.description)
            .finish()
    }
}

impl<S, Req, D> Resource<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync + Into<tower::BoxError>,
    D: PartialEq,
{
    fn new(resource: S) -> Self {
        let description = resource.describe();
        Self {
            inner: Buffer::new(resource, 32),
            semaphore: Arc::new(Semaphore::new(1)),
            description,
        }
    }

    async fn acquire(&self) -> (Buffer<S, Req>, OwnedSemaphorePermit) {
        let owned_semaphore = match self
            .semaphore
            .clone()
            .acquire_owned()
            .map_err::<AllocatorError, _>(Into::into)
            .await
        {
            Ok(sem) => sem,
            Err(_) => unreachable!("The semaphore will never close"),
        };

        (self.inner.clone(), owned_semaphore)
    }
}

impl<S, Req, D> Clone for Resource<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync + Into<tower::BoxError>,
    D: Clone + PartialEq,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            semaphore: self.semaphore.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Debug)]
pub struct AllocatorService<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync,
    D: Clone + PartialEq,
{
    num_times_called: usize,
    resources: Vec<Resource<S, Req, D>>,
}

impl<S, Req, D> AllocatorService<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync + Into<tower::BoxError>,
    D: PartialEq + Clone,
{
    pub fn new(resources: Vec<S>) -> Self {
        Self {
            num_times_called: 0,
            resources: resources.into_iter().map(Resource::new).collect(),
        }
    }
}

#[derive(Debug)]
pub enum AllocatorError {
    NoMatchingResource,
    SemaphoreProblem,
    ListenerProblem(String),
}

impl From<AcquireError> for AllocatorError {
    fn from(_: AcquireError) -> Self {
        Self::SemaphoreProblem
    }
}

impl Display for AllocatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AllocatorError {}

impl<S, Req, D> Service<D> for AllocatorService<S, Req, D>
where
    S: Service<Req> + Send + 'static,
    S: Describable<D>,
    Req: Send + 'static + Clone + DeserializeOwned,
    S::Response: Serialize + Send,
    S::Future: Send,
    S::Error: Send + Sync + Into<tower::BoxError>,
    D: Debug + Send + Clone + PartialEq + Sync + 'static,
{
    // The port where the allocated service waits for a connection.
    type Response = u16;
    type Error = AllocatorError;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: D) -> Self::Future {
        self.num_times_called += 1;

        // This will first filter any resources not matching the request of the caller.
        // Then we store futures in vector.
        // The futures will try to acquire the resource by getting the semaphore permit.
        //
        // By using `select_ok` on this (further below) we can "race to the finish".
        // I.e. we're fine with getting hold of the first matching resource, it does not
        // matter much which it actually is.
        let matching_services = self
            .resources
            .iter()
            .filter(|resource| resource.description == request)
            .cloned()
            .map(|resource| Box::pin(async move { Ok(resource.acquire().await) }))
            .collect::<Vec<_>>();

        let id = self.num_times_called;
        let label = format!("#{id}-{:?}", request);

        Box::pin(
            async move {
                if matching_services.is_empty() {
                    Err(AllocatorError::NoMatchingResource)
                } else {
                    match futures::future::select_ok(matching_services).await {
                        Ok(((resource, semaphore_permit), _)) => {
                            let (handle, port) = match mux_server::once("0.0.0.0:0", resource).await
                            {
                                Ok((handle, port)) => (handle, port),
                                Err(e) => todo!("Handle server error {:?}", e),
                            };

                            tokio::spawn(async move {
                                match handle.await {
                                    Ok(()) => debug!("Donny"),
                                    Err(e) => error!(?e, "Problem awaiting MuxServer"),
                                };
                                // This assures the semaphore was moved into this scope,
                                // and that it drops when the work is done.
                                drop(semaphore_permit)
                            });

                            Ok(port)
                        }
                        Err(e) => Err(e),
                    }
                }
            }
            .instrument(info_span!("handshake-fut", %label)),
        )
    }
}
