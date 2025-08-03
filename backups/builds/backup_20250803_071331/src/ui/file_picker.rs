use std::path::PathBuf;
use rfd::FileDialog;

pub struct FilePicker;

impl FilePicker {
    /// Open a file picker for selecting theme files (.yaml, .yml, .json)
    pub async fn pick_theme_file() -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("Theme Files", &["yaml", "yml", "json"])
            .add_filter("YAML Files", &["yaml", "yml"])
            .add_filter("JSON Files", &["json"])
            .set_title("Select Theme File")
            .pick_file()
    }

    /// Open a file picker for selecting background image files
    pub async fn pick_background_image() -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("Image Files", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
            .add_filter("PNG Files", &["png"])
            .add_filter("JPEG Files", &["jpg", "jpeg"])
            .add_filter("All Files", &["*"])
            .set_title("Select Background Image")
            .pick_file()
    }

    /// Open a file picker for saving theme files
    pub async fn save_theme_file(default_name: Option<&str>) -> Option<PathBuf> {
        let mut dialog = FileDialog::new()
            .add_filter("YAML Files", &["yaml"])
            .add_filter("JSON Files", &["json"])
            .set_title("Save Theme As");

        if let Some(name) = default_name {
            dialog = dialog.set_file_name(name);
        }

        dialog.save_file()
    }

    /// Get the user's themes directory
    pub fn get_themes_directory() -> Option<PathBuf> {
        dirs::config_dir().map(|config| config.join("warp").join("themes"))
    }

    /// Get the user's backgrounds directory
    pub fn get_backgrounds_directory() -> Option<PathBuf> {
        dirs::config_dir().map(|config| config.join("warp").join("backgrounds"))
    }

    /// Ensure directories exist
    pub fn ensure_directories() -> Result<(), std::io::Error> {
        if let Some(themes_dir) = Self::get_themes_directory() {
            std::fs::create_dir_all(&themes_dir)?;
        }

        if let Some(backgrounds_dir) = Self::get_backgrounds_directory() {
            std::fs::create_dir_all(&backgrounds_dir)?;
        }

        Ok(())
    }
}
