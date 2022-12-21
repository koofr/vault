use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use vault_core::eventstream::WebSocketClient;

#[wasm_bindgen(typescript_custom_section)]
const BROWSER_EVENTSTREAM_WEBSOCKET_DELEGATE: &'static str = r#"
export interface BrowserEventstreamWebSocketDelegate {
  open(
    url: string,
    onOpen: () => void,
    onMessage: (data: string) => void,
    onClose: () => void
  ): void;
  send(data: string): void;
  close(): void;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BrowserEventstreamWebSocketDelegate")]
    pub type BrowserEventstreamWebSocketDelegate;

    #[wasm_bindgen(structural, method)]
    pub fn open(
        this: &BrowserEventstreamWebSocketDelegate,
        url: String,
        on_open: &Closure<dyn Fn()>,
        on_message: &Closure<dyn Fn(String)>,
        on_close: &Closure<dyn Fn()>,
    );

    #[wasm_bindgen(structural, method)]
    pub fn send(this: &BrowserEventstreamWebSocketDelegate, data: String);

    #[wasm_bindgen(structural, method)]
    pub fn close(this: &BrowserEventstreamWebSocketDelegate);
}

pub struct BrowserEventstreamWebSocketClient {
    browser_eventstream_websocket_delegate: BrowserEventstreamWebSocketDelegate,
    on_open_closure: RefCell<Option<Closure<dyn Fn()>>>,
    on_message_closure: RefCell<Option<Closure<dyn Fn(String)>>>,
    on_close_closure: RefCell<Option<Closure<dyn Fn()>>>,
}

unsafe impl Send for BrowserEventstreamWebSocketClient {}
unsafe impl Sync for BrowserEventstreamWebSocketClient {}

impl BrowserEventstreamWebSocketClient {
    pub fn new(
        browser_eventstream_websocket_delegate: BrowserEventstreamWebSocketDelegate,
    ) -> BrowserEventstreamWebSocketClient {
        BrowserEventstreamWebSocketClient {
            browser_eventstream_websocket_delegate,
            on_open_closure: RefCell::new(None),
            on_message_closure: RefCell::new(None),
            on_close_closure: RefCell::new(None),
        }
    }
}

impl WebSocketClient for BrowserEventstreamWebSocketClient {
    fn open(
        &self,
        url: String,
        on_open: Box<dyn Fn() + Send + Sync + 'static>,
        on_message: Box<dyn Fn(String) + Send + Sync + 'static>,
        on_close: Box<dyn Fn() + Send + Sync + 'static>,
    ) {
        let on_open_closure = Closure::new(on_open);
        let on_message_closure = Closure::new(on_message);
        let on_close_closure = Closure::new(on_close);

        self.browser_eventstream_websocket_delegate.open(
            url,
            &on_open_closure,
            &on_message_closure,
            &on_close_closure,
        );

        self.on_open_closure.replace(Some(on_open_closure));
        self.on_message_closure.replace(Some(on_message_closure));
        self.on_close_closure.replace(Some(on_close_closure));
    }

    fn send(&self, data: String) {
        self.browser_eventstream_websocket_delegate.send(data)
    }

    fn close(&self) {
        self.browser_eventstream_websocket_delegate.close();

        self.on_open_closure.replace(None);
        self.on_message_closure.replace(None);
        self.on_close_closure.replace(None);
    }
}
