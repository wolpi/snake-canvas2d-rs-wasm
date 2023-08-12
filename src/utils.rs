use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
pub(crate) use log;

pub fn handle_js_error(result: Result<(), JsValue>) {
    if result.is_err() {
        log!("{}", result.err().unwrap().as_string().unwrap());
    }
}

pub fn random(min :u32, max :i32) -> i32 {
    js_sys::Math::floor(js_sys::Math::random() * max as f64 - min as f64) as i32 + min as i32
}
