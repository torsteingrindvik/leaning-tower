use std::{
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use async_bincode::AsyncBincodeStream;
use futures::{Future, TryFutureExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tower::multiplex::{self, MultiplexTransport};
use tower::{buffer::Buffer, Service, ServiceExt};

use crate::{
    resource_filter::Describable,
    shared,
    slab_store::SlabStore,
    tagged::{self, Request},
};

type HandshakeClient<HR> =
    shared::TokioTowerClient<tagged::Request<HR>, tagged::Response<shared::Port>>;

type EstablishedClient<EReq, EResp> =
    shared::TokioTowerClient<tagged::Request<EReq>, tagged::Response<EResp>>;

pub struct Client<EReq, EResp>
where
    EReq: Serialize,
    EResp: DeserializeOwned,
{
    // PINNOLINI
    inner: EstablishedClient<EReq, EResp>,
}

impl<EReq, EResp> tower::Service<EReq> for Client<EReq, EResp>
where
    EReq: Serialize + Send + 'static,
    EResp: DeserializeOwned + Send + 'static,
{
    type Response = tagged::Response<EResp>;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // self.inner.ready()
        todo!()
    }

    fn call(&mut self, req: EReq) -> Self::Future {
        // let port_fut = self
        //     .inner
        //     .ready()
        //     .and_then(|inner_ready| inner_ready.call(Request::new(req)));

        // let hmm = tagged::Request::new(req);
        // let resp = self.inner.ready()

        Box::pin(async move {
            // let response = port_fut.await.expect("TODO");
            // let client = Self::establish(response).await.expect("TODO");

            // Ok(client)
            todo!()
        })
    }
}


/// You should create one [Handshaker] per client machine
/// in order to get allocated resources.
///
/// Different tasks on each client may use [Clone] to get their own
/// handle.
/// This handle should then be used to get an [EstablishedClient].
#[derive(Clone)]
pub struct Handshaker<HR, EReq, EResp>
where
    HR: Serialize + Send + 'static + Clone,
{
    inner: Buffer<HandshakeClient<HR>, tagged::Request<HR>>,
    established_request: PhantomData<EReq>,
    established_response: PhantomData<EResp>,
}

impl<HR, EReq, EResp> Handshaker<HR, EReq, EResp>
where
    HR: Serialize + Send + 'static + Clone,
{
    // TODO: IntoSocketAddr or something
    pub async fn new(server: &str) -> Result<Self> {
        let tx = TcpStream::connect(server).await?;
        let tx = AsyncBincodeStream::from(tx).for_async();

        let client = multiplex::Client::new(MultiplexTransport::new(tx, SlabStore::default()));
        let client = Buffer::new(client, 32);

        Ok(Self {
            inner: client,
            established_request: PhantomData::default(),
            established_response: PhantomData::default(),
        })
    }

    async fn establish(response: tagged::Response<shared::Port>) -> Result<Client<EReq, EResp>>
    where
        EReq: Serialize + Send + 'static + Clone,
        EResp: DeserializeOwned + Send + 'static,
    {
        // TODO: Fix addr
        let tx = TcpStream::connect(format!("127.0.0.1:{}", response.inner)).await?;
        let tx = AsyncBincodeStream::from(tx).for_async();

        let client = multiplex::Client::new(MultiplexTransport::new(tx, SlabStore::default()));
        let client = Client { inner: client };

        Ok(client)
    }
}

impl<HR, EReq, EResp> tower::Service<HR> for Handshaker<HR, EReq, EResp>
where
    HR: Serialize + Send + 'static + Clone,
    EReq: Serialize + Send + 'static,
    EResp: DeserializeOwned + Send + 'static,
{
    type Response = Client<EReq, EResp>;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // self.inner.ready()
        todo!()
    }

    fn call(&mut self, req: HR) -> Self::Future {
        todo!()
        // let port_fut = self
        //     .inner
        //     .ready()
        //     .and_then(|inner_ready| inner_ready.call(Request::new(req)));

        // Box::pin(async move {
        //     let response = port_fut.await.expect("TODO");
        //     let client = Self::establish(response).await.expect("TODO");

        //     Ok(client)
        // })
    }
}

// pub async fn connect<Request>() -> Result<HandshakeClient<Request>>
// where
//     Request: Serialize + Send + 'static,
// {
//     let tx = TcpStream::connect(shared::SERVER_BIND_ADDR).await?;
//     let tx = AsyncBincodeStream::from(tx).for_async();

//     let client = multiplex::Client::new(MultiplexTransport::new(tx, SlabStore::default()));

//     Ok(client)
// }

// pub async fn established_call(
//     client: &mut EstablishedClient,
//     request: shared::MainRequest,
// ) -> Result<shared::MainResponse> {
//     Ok(client
//         .ready()
//         .await
//         .map_err(|e| anyhow::anyhow!(e))?
//         .call(request)
//         .await
//         .map_err(|e| anyhow::anyhow!(e))?)
// }
