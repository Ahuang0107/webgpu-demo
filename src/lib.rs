mod app;
mod render;

pub use app::*;
pub use render::*;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
