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
    (js_sys::Math::floor(js_sys::Math::random() * max as f64 - min as f64) as u32 + min) as i32
}

pub fn format_duration(duration :u32) -> String {
    let duration_in_secs = duration / 1000;
    let minutes = duration_in_secs / 60;
    let seconds = duration_in_secs % 60;
    return format!("{:02}:{:02}", minutes, seconds)
}
