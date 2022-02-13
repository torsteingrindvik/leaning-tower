use std::pin::Pin;

use slab::Slab;
use tokio_tower::multiplex::TagStore;

use crate::tagged;

pub struct SlabStore(Slab<()>);

impl SlabStore {
    pub fn new() -> Self {
        Self(Slab::new())
    }
}

impl Default for SlabStore {
    fn default() -> Self {
        Self::new()
    }
}

impl<Request, Response> TagStore<tagged::Request<Request>, tagged::Response<Response>> for SlabStore
where
    Request: Clone,
{
    type Tag = usize;
    fn assign_tag(mut self: Pin<&mut Self>, request: &mut tagged::Request<Request>) -> usize {
        let tag = self.0.insert(());
        request.set_tag(tag);
        tag
    }
    fn finish_tag(mut self: Pin<&mut Self>, response: &tagged::Response<Response>) -> usize {
        let tag = response.tag();
        self.0.remove(tag);
        tag
    }
}
