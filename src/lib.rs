mod utils;
mod textdisplay;
mod highscore;
mod game;

use crate::game::Game;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;


// state.
// Currently there is no better option in wasm than to have a global variable as static mut.
// That requires unsafe blocks, which is ok as wasm is single threaded.
static mut GAME: Game = Game::default();


#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    utils::set_panic_hook();
    log!("starting in rust");

    let document = web_sys::window().unwrap().document().unwrap();
    register_event_listeners(&document)?;
    highscore::print_highscores(None);

    Ok(())
}

fn register_event_listeners(document: &web_sys::Document) -> Result<(), JsValue> {
    log!("register_event_listeners()");

    register_event_listener_create(document)?;
    register_event_listener_input_keyboard(document)?;
    register_event_listeners_input_touch(document)?;
    
    Ok(())
}

fn register_event_listener_create(document: &web_sys::Document) -> Result<(), JsValue> {
    let callback = Closure::wrap(Box::new(|_e: web_sys::Event| {

        unsafe {
            if GAME.is_over() {
               create_game();
            }
        }

    }) as Box<dyn FnMut(_)>);

    document.get_element_by_id("create").unwrap()
        .add_event_listener_with_callback("click", &callback.as_ref().unchecked_ref())?;

    callback.forget();

    Ok(())
}

fn register_event_listener_input_keyboard(document: &web_sys::Document) -> Result<(), JsValue> {
    let callback_keydown = Closure::wrap(Box::new(|e: web_sys::KeyboardEvent| {
        //log!("e.key_code(): {}", e.key_code());
        unsafe {
            if !GAME.is_over() {
                e.prevent_default();
                match e.key_code() {
                    0x41 => GAME.set_input('a'),
                    0x53 => GAME.set_input('s'),
                    0x44 => GAME.set_input('d'),
                    0x57 => GAME.set_input('w'),
                    32 => GAME.set_input(' '),
                    _ => GAME.set_input(game::DEFAULT_INPUT),
                }
                GAME.set_pressed(true);
            }
        }
    }) as Box<dyn FnMut(_)>);

    let callback_keyup = Closure::wrap(Box::new(|e: web_sys::KeyboardEvent| {
        unsafe {
            if !GAME.is_over() {
                e.prevent_default();
                GAME.set_pressed(false);
            }
        }
    }) as Box<dyn FnMut(_)>);

    let body = document.get_element_by_id("body").unwrap();
    body.add_event_listener_with_callback("keydown", &callback_keydown.as_ref().unchecked_ref())?;
    body.add_event_listener_with_callback("keyup", &callback_keyup.as_ref().unchecked_ref())?;

    callback_keydown.forget();
    callback_keyup.forget();

    Ok(())
}

fn register_event_listeners_input_touch(document: &web_sys::Document) -> Result<(), JsValue> {
    register_event_listener_input_touch(document, 'w', "touch-up")?;
    register_event_listener_input_touch(document, 'a', "touch-left")?;
    register_event_listener_input_touch(document, 's', "touch-down")?;
    register_event_listener_input_touch(document, 'd', "touch-right")?;
    register_event_listener_input_touch(document, ' ', "touch-pause")?;
    Ok(())
}

fn register_event_listener_input_touch(document: &web_sys::Document, key :char, id :&str) -> Result<(), JsValue> {
    let callback = Closure::wrap(Box::new(move || {
        unsafe {
            GAME.set_input(key);
        }
    }) as Box<dyn FnMut()>);

    document.get_element_by_id(id).unwrap()
        .add_event_listener_with_callback("click", &callback.as_ref().unchecked_ref())?;

    callback.forget();

    Ok(())
}

pub fn create_game() {
    log!("creating game");
    let document = web_sys::window().unwrap().document().unwrap();

    let width_element = document.get_element_by_id("width").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let height_element = document.get_element_by_id("height").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let block_size_element = document.get_element_by_id("block-size").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let grid_element = document.get_element_by_id("grid").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let touch_mode_element = document.get_element_by_id("touch-mode").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let game_mode_element = document.get_element_by_id("game-mode").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    let name_element = document.get_element_by_id("name").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    log!("  got parameter elements");

    let width = width_element.value_as_number() as u32;
    let height = height_element.value_as_number() as u32;
    let block_size = block_size_element.value_as_number() as u32;
    let draw_grid = grid_element.checked();
    let touch_mode = touch_mode_element.checked();
    let game_mode_str = game_mode_element.value();
    let game_mode = if game_mode_str.starts_with("Fast") {
        game::GameMode::FAST
    } else {
        game::GameMode::LONG
    };
    let name = name_element.value();
    log!("  got parameter values");

    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    log!("  got canvas context");

    unsafe {
        GAME.set_state(width, height, block_size, draw_grid, touch_mode, game_mode, &name, context);
    }

    start_world_loop();
}

struct RenderLoop {
    animation_id: Option<i32>,
    closure: Option<Closure<dyn FnMut(u32)>>,
}

const EXPECT_MSG: &str = "cannot request animation frame";

fn start_world_loop() {
    // see https://users.rust-lang.org/t/wasm-web-sys-how-to-use-window-request-animation-frame-resolved/20882
    let render_loop: Rc<RefCell<RenderLoop>> = Rc::new(RefCell::new(RenderLoop{animation_id: None, closure: None}));
    {
        let closure: Closure<dyn FnMut(u32)> = {
            let render_loop = render_loop.clone();
            Closure::wrap(Box::new(move |timestamp :u32| {

                unsafe {
                    let continue_rendering = GAME.world_loop_contents(timestamp);
                    if !continue_rendering {
                        return;
                    }
                }

                let window = web_sys::window().unwrap();
                let mut render_loop = render_loop.borrow_mut();
                render_loop.animation_id = if let Some(ref closure) = render_loop.closure {
                    Some(window.request_animation_frame(closure.as_ref().unchecked_ref()).expect(EXPECT_MSG))
                } else {
                    None
                }
            }))
        };
        let window = web_sys::window().unwrap();
        let mut render_loop = render_loop.borrow_mut();
        render_loop.animation_id = Some(window.request_animation_frame(closure.as_ref().unchecked_ref()).expect(EXPECT_MSG));
        render_loop.closure = Some(closure);
    }
}
