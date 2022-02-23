use anyhow::Result;
use futures::Future;
use leaning_tower::resource_filter::Describable;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tower::Service;

use crate::data_discarder_types::{self, Action, DataDiscarderVariant};

#[derive(Debug)]
pub struct DataDiscarder {
    pub variant: DataDiscarderVariant,
    pub slowness_ms: usize,
}

impl DataDiscarder {
    pub fn new(variant: DataDiscarderVariant) -> Self {
        Self {
            variant,
            slowness_ms: match variant {
                DataDiscarderVariant::Fast => 5,
                DataDiscarderVariant::Medium => 25,
                DataDiscarderVariant::Slow => 100,
            },
        }
    }
}

impl Describable<DataDiscarderVariant> for DataDiscarder {
    fn describe(&self) -> DataDiscarderVariant {
        self.variant
    }
}

impl Service<Action> for DataDiscarder {
    type Response = data_discarder_types::Response;
    type Error = tower::BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, request: Action) -> Self::Future {
        let Action { payload } = request;
        let delay_ms = self.slowness_ms;

        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;
            Ok(data_discarder_types::Response {
                payload_size: payload.len(),
            })
        })
    }
}
