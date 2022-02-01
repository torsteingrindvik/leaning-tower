use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::info;

#[derive(Debug)]
pub struct VocalSemaphore {
    semaphore: Arc<Semaphore>,
    _permit: Option<OwnedSemaphorePermit>,
    label: String,
}

impl VocalSemaphore {
    pub fn new(semaphore: Arc<Semaphore>, label: String) -> Self {
        Self {
            semaphore,
            _permit: None,
            label,
        }
    }

    pub async fn acquire(&mut self) -> Result<()> {
        info!(%self.label, "Acquiring");
        self._permit = Some(self.semaphore.clone().acquire_owned().await?);
        info!(%self.label, "Acquired");

        Ok(())
    }
}

impl Drop for VocalSemaphore {
    fn drop(&mut self) {
        info!("Dropping `{}`", self.label);
    }
}
