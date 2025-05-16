mod app;
mod render;
mod start;

pub use app::*;
pub use render::*;
pub use start::*;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub const WEB_CANVAS_CONTAINER: &'static str = "wgpu-app-container";
