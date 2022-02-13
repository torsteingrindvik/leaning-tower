
// use std::{
//     fmt::{Debug, Display},
//     pin::Pin,
//     sync::Arc,
//     task::{Context, Poll},
// };

// // use anyhow::Result;
// use futures::TryFutureExt;
// use futures_core::Future;
// use tokio::sync::{AcquireError, Semaphore};
// use tower::{buffer::Buffer, Service};
// use tracing::{info_span, Instrument};

// use crate::{resource_filter::Describable, shared, tagged};

// #[derive(Debug)]
// pub struct Resource<S, R, D>
// where
//     S: Service<R> + Send + 'static,
//     S: Describable<D, R>,
//     R: Send + 'static,
//     S::Future: Send + Debug,
//     S::Error: Send + Sync,
// {
//     inner: Buffer<S, R>,
//     description: D,
//     semaphore: Arc<Semaphore>,
// }

// impl<S, R, D> Resource<S, R, D>
// where
//     S: Service<R> + Send + 'static,
//     S: Describable<D, R>,
//     R: Send + 'static,
//     S::Future: Send + Debug,
//     S::Error: Send + Sync + Into<tower::BoxError>,
//     // impl<S> Resource<S> {
// {
//     fn new(resource: S) -> Self {
//         Self {
//             inner: Buffer::new(resource, 32),
//             semaphore: Arc::new(Semaphore::new(1)),
//             description: resource.describe(),
//         }
//     }
// }

// impl<S, R, D> Clone for Resource<S, R, D>
// where
//     S: Service<R> + Send + 'static,
//     S: Describable<D, R>,
//     R: Send + 'static,
//     S::Future: Send + Debug,
//     S::Error: Send + Sync + Into<tower::BoxError>,
// {
//     fn clone(&self) -> Self {
//         Self {
//             inner: self.inner.clone(),
//             semaphore: self.semaphore.clone(),
//             description: self.description.clone(),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct AllocatorService<S, R, D>
// where
//     S: Service<R> + Send + 'static,
//     S: Describable<D, R>,
//     R: Send + 'static,
//     S::Future: Send + Debug,
//     S::Error: Send + Sync,
// {
//     num_times_called: usize,
//     resources: Vec<Resource<S, R, D>>,
// }

// // #[derive(Debug)]
// // pub struct AllocatorService<S> {
// //     num_times_called: usize,
// //     resources: Vec<Resource<S>>,
// // }

// impl<S, R, D> AllocatorService<S, R, D>
// where
//     S: Service<R> + Send + 'static,
//     S: Describable<D, R>,
//     R: Send + 'static,
//     S::Future: Send + Debug,
//     S::Error: Send + Sync + Into<tower::BoxError>,
//     //     impl<S> AllocatorService<S>
//     // where
//     //     S: Service<R> + Send + 'static,
//     //     R: Send + 'static,
//     //     S::Future: Send + Debug,
//     //     S::Error: Send + Sync + Into<tower::BoxError>,
// {
//     pub fn new(resources: Vec<S>) -> Self {
//         Self {
//             num_times_called: 0,
//             resources: resources.into_iter().map(Resource::new).collect(),
//         }
//     }
// }

// #[derive(Debug)]
// pub enum AllocatorError {
//     NoMatchingResource,
//     SemaphoreProblem,
// }

// impl From<AcquireError> for AllocatorError {
//     fn from(e: AcquireError) -> Self {
//         Self::SemaphoreProblem
//     }
// }

// impl Display for AllocatorError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

// impl std::error::Error for AllocatorError {}

// // impl<S, Req> Service<tagged::TagRequest<Req>> for AllocatorService<S> where
// //     S: FilteredBy<Req> + Sync + Send,
// //     S: Service<R> + Send + 'static,
// //     R: Send + 'static,
// // S::Future: Send + Debug,
// //     S::Error: Send + Sync,
// // Req: Debug,
// impl<S, D, Request> Service<tagged::TagRequest<Request>> for AllocatorService<S, Request, D>
// where
//     S: Service<R> + Send + 'static,
//     S: Describable<D, Request>,
//     R: Send + 'static,
//     S::Future: Send + Debug,
//     S::Error: Send + Sync + Into<tower::BoxError>,
//     Request: Debug,
// {
//     type Response = tagged::TagResponse<shared::Port>;
//     type Error = AllocatorError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, request: tagged::TagRequest<Request>) -> Self::Future {
//         self.num_times_called += 1;

//         let matching_services = self
//             .resources
//             .iter()
//             .filter(|resource| S::matches(resource.description, request.inner))
//             .cloned()
//             .map(|resource| {
//                 Box::pin(async move {
//                     resource
//                         .semaphore
//                         .acquire_owned()
//                         .map_err::<AllocatorError, _>(Into::into)
//                         .await
//                 })
//             })
//             .collect::<Vec<_>>();

//         let id = self.num_times_called;
//         let label = format!("#{id}-{:?}", request);
//         let label_move = label.clone();

//         Box::pin(
//             async move {
//                 if matching_services.is_empty() {
//                     Err(AllocatorError::NoMatchingResource)
//                 } else {
//                     match futures::future::select_ok(matching_services).await {
//                         Ok(_) => todo!(),
//                         Err(_) => todo!(),
//                     }
//                 }

//                 // match get_first.await {
//                 //     Ok(thing) => todo!(),
//                 //     Err(e) => todo!(),
//                 // }
//                 // match res {
//                 //     Ok(semaphore) => {
//                 //         // let mut sem = VocalSemaphore::new(semaphore, label_move.clone());
//                 //         // sem.acquire().instrument(info_span!("acquire")).await?;

//                 //         info!(%label_move, "Making new transport for acquired");
//                 //         server::spawn_new_transport(req, server_service::MainService::new(id, sem))
//                 //             .instrument(info_span!("spawn-transport"))
//                 //             .await
//                 //     }
//                 //     Err(e) => Err(e),
//                 // }
//             }
//             .instrument(info_span!("handshake-fut", %label)),
//         )
//     }
// }
