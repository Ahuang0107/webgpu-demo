mod audio;
mod fps;
mod framework;
mod render;
mod start;

pub use audio::*;
pub use fps::*;
pub use framework::*;
pub use render::*;
pub use start::*;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub const WEB_CANVAS_CONTAINER: &'static str = "wgpu-app-container";
