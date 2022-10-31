#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod hex_pattern;
mod parsing;
mod rendering;
pub use app::HexeditApp;