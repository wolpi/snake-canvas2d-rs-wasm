use wasm_bindgen::prelude::*;
use web_sys::Document;

fn show_value(document :&Document, element_id :&str, val :u32) {
    let element = document.get_element_by_id(element_id).unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
    element.set_text_content(Some(&val.to_string()));
}

pub fn update_text_display(score :u32, speed :u32) {
    let document = web_sys::window().unwrap().document().unwrap();
    show_value(&document, "score", score);
    show_value(&document, "current-speed", speed);
}

pub fn set_background_colour(colour :&str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.get_element_by_id("body").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
    let style = element.style();
    style.set_css_text(&("background-color: ".to_owned() + colour + ";"));
}
