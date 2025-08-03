//! Icon definitions for the application.

use iced::Font;

// Font Awesome font - you'd need to include the font file in your app
// For now, we'll use Unicode symbols that are widely supported
pub const ICON_FONT: Font = Font::DEFAULT;

// Modern Unicode symbols that work across platforms
pub const DOWNLOAD: char = '⬇';     // Download arrow
pub const CODE: char = '⚡';        // Code/lightning for development 
pub const DEPLOY: char = '🚀';      // Rocket for deployment
pub const AGENT: char = '🤖';       // Robot for AI agent
pub const COPY: char = '📋';        // Clipboard
pub const SHARE: char = '🔗';       // Link for sharing
pub const BOOKMARK: char = '🔖';    // Bookmark
pub const BOOKMARK_FILLED: char = '📌'; // Filled bookmark
pub const TERMINAL: char = '⌨';     // Terminal/keyboard
pub const SETTINGS: char = '⚙';     // Gear for settings
pub const FOLDER: char = '📁';      // Folder
pub const FILE: char = '📄';        // Document
pub const SUCCESS: char = '✅';     // Success checkmark
pub const ERROR: char = '❌';       // Error cross
pub const WARNING: char = '⚠';     // Warning triangle
pub const INFO: char = 'ℹ';        // Information

// Helper function to get appropriate icon based on context
pub fn get_status_icon(exit_code: Option<i32>) -> char {
    match exit_code {
        Some(0) => SUCCESS,
        Some(_) => ERROR,
        None => INFO,
    }
}

// Helper function to get bookmark icon based on state
pub fn get_bookmark_icon(bookmarked: bool) -> char {
    if bookmarked { BOOKMARK_FILLED } else { BOOKMARK }
}
