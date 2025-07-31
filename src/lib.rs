//! The main library for the Warp terminal application.
//!
//! This crate contains the core logic for the terminal, including block management,
//! command execution, UI components, and theme handling.

// Module declarations
pub mod app;
pub mod ui;
pub mod model;
pub mod executor;
pub mod theme;
pub mod utils;
pub mod keyset;
pub mod input;
pub mod persistence;
pub mod editor;
pub mod ai;
pub mod handlers;

// Re-export key components for the binary and for clarity.
pub use app::terminal::WarpTerminal;
pub use app::terminal::Message;
pub use model::block::{Block, BlockManager};
