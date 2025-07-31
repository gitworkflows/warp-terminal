pub mod text_editor;
pub mod syntax_highlighter;
pub mod completion_engine;
pub mod editor_state;
pub mod keybindings;

pub use text_editor::ModernTextEditor;
pub use editor_state::{EditorState, EditorAction};
pub use syntax_highlighter::{SyntaxHighlighter, HighlightedSpan};
pub use completion_engine::{CompletionEngine, CompletionItem};
pub use keybindings::{KeybindingManager, EditorKeybinding};
