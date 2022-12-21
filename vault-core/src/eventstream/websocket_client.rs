pub trait WebSocketClient {
    fn open(
        &self,
        url: String,
        on_open: Box<dyn Fn() + Send + Sync + 'static>,
        on_message: Box<dyn Fn(String) + Send + Sync + 'static>,
        on_close: Box<dyn Fn() + Send + Sync + 'static>,
    );
    fn send(&self, data: String);
    fn close(&self);
}
