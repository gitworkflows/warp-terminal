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
pub mod monitoring;

// Additional modules
pub mod agent_mode_eval;
pub mod asset_macro;
pub mod command;
pub mod graphql;
pub mod inspector;
pub mod languages;
pub mod lpc;
pub mod markdown_parser;
pub mod mcq;
pub mod natural_language_detection;
pub mod shell;
pub mod string_offset;
pub mod sum_tree;
pub mod syntax_tree;
pub mod virtual_fs;
pub mod watcher;
pub mod websocket;

// Command signatures v2 module (using hyphenated directory name)
#[path = "command-signatures-v2/mod.rs"]
pub mod command_signatures_v2;

// Re-export key components for the binary and for clarity.
pub use app::terminal::WarpTerminal;
pub use app::terminal::Message;
pub use model::block::{Block, BlockManager};
