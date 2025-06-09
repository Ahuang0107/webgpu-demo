mod app;
mod assets;
mod audio;
mod easing;
#[cfg(feature = "editor_mode")]
mod egui_render;
mod fps;
mod framework;
mod input;
mod render;
mod utils;

pub use app::*;
pub use audio::*;
pub use easing::*;
pub use fps::*;
pub use framework::*;
pub use render::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub const WEB_CANVAS_CONTAINER: &'static str = "wgpu-app-container";

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    run::<AppData>(PKG_NAME).expect("TODO: panic message");
}
