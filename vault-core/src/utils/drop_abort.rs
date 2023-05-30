use futures::stream::AbortHandle;

pub struct DropAbort(pub AbortHandle);

impl Drop for DropAbort {
    fn drop(&mut self) {
        self.0.abort()
    }
}
