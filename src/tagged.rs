use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request<T> {
    pub(crate) inner: T,
    tag: usize,
}

impl<T> Request<T> where T: Clone {
    pub fn new(request: T) -> Self {
        Self {
            inner: request,
            tag: 0,
        }
    }

    pub fn set_tag(&mut self, tag: usize) {
        self.tag = tag;
    }

    /// Extract the inner request.
    pub fn inner(self) -> T {
        self.inner
    }

    /// Clone the inner request.
    pub fn clone_inner(&self) -> T {
        self.inner.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub(crate) inner: T,
    tag: usize,
}

impl<T> Response<T> {
    /// Get a reference to the tag response's tag.
    pub fn tag(&self) -> usize {
        self.tag
    }

    pub fn new<R>(request: Request<R>, response: T) -> Self {
        Self {
            inner: response,
            tag: request.tag,
        }
    }

    /// Extract the inner response.
    pub fn inner(self) -> T {
        self.inner
    }
}
