use std::panic;

use console_log;
use log::Level;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(js_name = initConsole)]
pub fn init_console() {
    panic::set_hook(Box::new(panic_hook));

    console_log::init_with_level(Level::Debug).unwrap();
}

// Copied from
// https://github.com/rustwasm/console_error_panic_hook/blob/4dc30a5448ed3ffcfb961b1ad54d000cca881b84/src/lib.rs

#[wasm_bindgen]
extern "C" {
    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

fn panic_hook(info: &panic::PanicHookInfo) {
    let mut msg = info.to_string();

    // Add the error stack to our message.
    //
    // This ensures that even if the `console` implementation doesn't
    // include stacks for `console.error`, the stack is still available
    // for the user. Additionally, Firefox's console tries to clean up
    // stack traces, and ruins Rust symbols in the process
    // (https://bugzilla.mozilla.org/show_bug.cgi?id=1519569) but since
    // it only touches the logged message's associated stack, and not
    // the message's contents, by including the stack in the message
    // contents we make sure it is available to the user.
    msg.push_str("\n\nStack:\n\n");
    let e = Error::new();
    let stack = e.stack();
    msg.push_str(&stack);

    // Safari's devtools, on the other hand, _do_ mess with logged
    // messages' contents, so we attempt to break their heuristics for
    // doing that by appending some whitespace.
    // https://github.com/rustwasm/console_error_panic_hook/issues/7
    msg.push_str("\n\n");

    // Finally, log the panic with `console.error`!
    console::error_1(&msg.into());
}
