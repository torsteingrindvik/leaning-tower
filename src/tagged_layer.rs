use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;
use tower::{
    util::{BoxService, MapRequest, MapRequestLayer, MapResponse},
    Service, ServiceBuilder, ServiceExt,
};

use crate::tagged;

// use std::{
//     pin::Pin,
//     task::{Context, Poll},
// };

// use futures::Future;
// use pin_project_lite::pin_project;
// use tower::{Layer, Service};

// use crate::tagged;

// pub struct TagLayer {}

// impl<S> Layer<S> for TagLayer {
//     type Service = TagService<S>;

//     fn layer(&self, inner: S) -> Self::Service {
//         todo!()
//     }
// }

// pub struct TagService<S> {
//     service: S,
// }

// pin_project! {
//     #[derive(Debug)]
//     pub struct ResponseFuture<T> {
//         #[pin]
//         response: T
//     }
// }

// impl<T> ResponseFuture<T> {
//     pub(crate) fn new(response: T) -> Self {
//         Self { response }
//     }
// }

// impl<F, T, E> Future for ResponseFuture<F>
// where
//     F: Future<Output = Result<tagged::TagResponse<T>, E>>,
//     E: Into<tower::BoxError>,
// {
//     type Output = Result<T, tower::BoxError>;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();

//         match this.response.poll(cx) {
//             Poll::Ready(v) => {
//                 return Poll::Ready(v.map(|tag_response| tag_response.inner).map_err(Into::into))
//             }
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }

// impl<S, Request> Service<Request> for TagService<S>
// where
//     S: Service<Request>,
//     S::Error: Into<tower::BoxError>,
// {
//     type Response = S::Response;
//     type Error = tower::BoxError;
//     type Future = ResponseFuture<S::Future>;

//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn call(&mut self, req: Request) -> Self::Future {
//         let response = self.service.call(req);
//         ResponseFuture::new(response)
//     }
// }

// About MapRequest:
//
// S: Service<R2>,
// F: FnMut(R1) -> R2,
//
// So we need a function which takes a request and outputs tagged::Request.

// About MapResponse:
//
// F: FnOnce(S::Response) -> Response + Clone
//
// So we need a function which takes a tagged::Response and extracts it.

// pub struct TagService<S, Req, Resp, F1, F2> {
//     service: MapResponse<MapRequest<S, F1>, F2>,
//     mapped_request: PhantomData<Req>,
//     mapped_response: PhantomData<Resp>,
// }
// pub struct TagService<S, Req, Resp> {
//     service: MapResponse<MapRequest<S, fn(Req) -> tagged::Request<Req>>, fn(tagged::Response<Resp>) -> Resp>,
//     mapped_request: PhantomData<Req>,
//     mapped_response: PhantomData<Resp>,
// }

fn map_response<Resp>(response: tagged::Response<Resp>) -> Resp {
    response.inner()
}

// // impl<S, Req, Resp, F1, F2> TagService<S, Req, Resp, F1, F2> {
// impl<S, Req, Resp> TagService<S, Req, Resp> {

//     pub fn new(service: S) -> Self {
//         // let map_request = |t: Req| tagged::Request::new(t);
//         // let map_response = |t: tagged::Response<Resp>| t.inner();

//         let service = MapRequest::new(service, map_request::<Req>);
//         let service = MapResponse::new(service, map_response::<Resp>);

//         Self {
//             service,
//             mapped_request: PhantomData::default(),
//             mapped_response: PhantomData::default(),
//         }
//     }
// }

// impl Service<

// struct Bleh<S, T>
// where
//     S: Service<T>,
// {
//     inner: BoxService<T, S::Response, S::Error>,
// }

// impl<S, T> Bleh<S, T>
// where
//     S: Service<T> + Send,
//     S::Future: Send
// {
//     fn new(service: S) -> Self {
//         let service = ServiceBuilder::new()
//             .map_request(|request: tagged::Request<T>| request.inner())
//             .map_response(|response: tagged::Response<S::Response>| response.inner())
//             .service(service);

//         let service = BoxService::new(service);

//         // let service = MapRequest::new(service, map_request::<T>);
//         // let service = MapResponse::new(service, map_response::<S::Response>);
//         // let service = BoxService::new(service);

//         Self { inner: service }
//     }
// }

// fn map_request<Req>(request: Req) -> tagged::Request<Req> {
//     tagged::Request::new(request)
// }

// struct TagService<S, Req>
// where
//     S: Service<Req>,
// {
//     tagged: MapRequest<S, fn(Req) -> tagged::Request<Req>>,
// }

// impl<S, Req> TagService<S, Req>
// where
//     S: Service<Req>,
// {
//     fn new(inner: S) -> Self {
//         let tagged = MapRequest::new(inner, map_request::<Req>);
//         Self { tagged }
//     }
// }

// fn tag_service<S, Req, Resp>(service: S) -> MapRequest<S, fn(Req) -> tagged::Request<Req>>
// where
//     S: Service<Req>,
// {
//     // let service = MapResponse::new(service, map_response::<Resp>);

//     let service = MapRequest::new(service, map_request::<Req>);
//     service
// }

// struct TagsRequests<S, T>
// where
//     S: Service<tagged::Request<T>>,
// {
//     inner: BoxService<T, S::Response, S::Error>,
// }

// impl<S, T> TagsRequests<S, T>
// where
//     S: Service<tagged::Request<T>> + Send + 'static,
//     S::Future: Send + 'static,
// {
//     fn new(service: S) -> Self {
//         // let inner = service.map_response(|response: tagged::Response<S::Response>| response.inner());

//         Self {
//             inner: service.map_request(|req| tagged::Request::new(req)).boxed(),
//         }
//     }
// }

// impl<S, T> Service<T> for TagsRequests<S, T>
// where
//     S: Service<tagged::Request<T>> + Send + 'static,
//     S::Future: Send + 'static,
//     S::Error: std::fmt::Debug + 'static,
//     S::Response: 'static,
// {
//     type Response = S::Response;
//     type Error = S::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn call(&mut self, req: T) -> Self::Future {
//         let fut = self.inner.call(req);

//         Box::pin(async move { Ok(fut.await.unwrap()) })
//     }
// }

// struct Hmm<S> {
//     inner: S,
// }

// impl<S, T> Service<tagged::Request<T>> for Hmm<S>
// where
//     S: Service<T>,
// {
//     type Response = tagged::Response<S::Response>;
//     type Error = tower::BoxError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn call(&mut self, req: tagged::Request<T>) -> Self::Future {
//         todo!()
//     }
// }

/*
    What we want:
        The user supplies a service.
        S<T, U> where S: Service<T>, S::Response = U

    We want to hijack this


    Or maybe we just
*/

// struct HereWeGo<S, T, U> {
//     inner: S,
//     request: PhantomData<T>,
//     response: PhantomData<U>,
// }

// struct HereWeFuture<U> {
//     response: PhantomData<U>,
// }

// impl<U> Future for HereWeFuture<U> {
//     type Output = Result<tagged::Response<U>, tower::BoxError>;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         todo!()
//     }
// }

// impl<S, T, U> Service<tagged::Request<T>> for HereWeGo<S, T, U> {
//     type Response = tagged::Response<U>;
//     type Error = tower::BoxError;
//     type Future = HereWeFuture<U>;

//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn call(&mut self, req: tagged::Request<T>) -> Self::Future {
//         todo!()
//     }
// }

// impl<S, T, U> Service<T> for HereWeGo<S, T, U> {
//     type Response = tagged::;
//     type Error;
//     type Future;

//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         todo!()
//     }

//     fn call(&mut self, req: T) -> Self::Future {
//         todo!()
//     }
// }
