use crate::ui::modern_components::WarpUI;
use crate::ui::file_picker;
use iced::widget::{button, column, container, row, text, Space, slider};
use iced::{theme, Alignment, Background, Border, Color, Element, Length, Shadow, Vector};
use std::path::PathBuf;

pub struct ThemeSelector {
    pub available_themes: Vec<ThemeInfo>,
    pub current_theme: String,
    pub search_query: String,
    pub category_filter: ThemeCategory,
    pub preview_theme: Option<String>,
    pub favorites: Vec<String>,
    pub recently_used: Vec<String>,
    pub background_image: Option<PathBuf>,
    pub background_opacity: f32,
    pub background_blur: f32,
    pub show_background_settings: bool,
}

#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub name: String,
    pub display_name: String,
    pub category: ThemeCategory,
    pub description: String,
    pub author: Option<String>,
    pub colors: ThemeColorPreview,
    pub is_favorite: bool,
    pub last_used: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeCategory {
    All,
    Dark,
    Light,
    HighContrast,
    Colorful,
    Minimal,
    Special,
    Custom,
}

#[derive(Debug, Clone)]
pub struct ThemeColorPreview {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub primary: Color,
    pub secondary: Color,
}

impl ThemeSelector {
    pub fn new() -> Self {
        let mut selector = Self {
            available_themes: Vec::new(),
            current_theme: "default_dark".to_string(),
            search_query: String::new(),
            category_filter: ThemeCategory::All,
            preview_theme: None,
            favorites: Vec::new(),
            recently_used: Vec::new(),
            background_image: None,
            background_opacity: 0.8,
            background_blur: 0.0,
            show_background_settings: false,
        };

        selector.load_default_themes();
        selector
    }

    fn load_default_themes(&mut self) {
        // Add some default themes for demonstration
        let default_themes = vec![
            ("dark_default", "Dark Default", ThemeCategory::Dark, "Default dark theme"),
            ("light_default", "Light Default", ThemeCategory::Light, "Default light theme"),
            ("high_contrast", "High Contrast", ThemeCategory::HighContrast, "High contrast theme"),
            ("dracula", "Dracula", ThemeCategory::Dark, "Popular dark theme"),
            ("monokai", "Monokai", ThemeCategory::Dark, "Classic dark theme"),
            ("github_light", "GitHub Light", ThemeCategory::Light, "GitHub's light theme"),
            ("solarized_dark", "Solarized Dark", ThemeCategory::Dark, "Solarized dark variant"),
            ("nord", "Nord", ThemeCategory::Dark, "Arctic blue theme"),
        ];

        for (name, display_name, category, description) in default_themes {
            let colors = self.generate_sample_colors(&category);
            
            self.available_themes.push(ThemeInfo {
                name: name.to_string(),
                display_name: display_name.to_string(),
                category,
                description: description.to_string(),
                author: Some("Warp Team".to_string()),
                colors,
                is_favorite: false,
                last_used: None,
            });
        }

        // Sort themes by name
        self.available_themes.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    }

    fn generate_sample_colors(&self, category: &ThemeCategory) -> ThemeColorPreview {
        match category {
            ThemeCategory::Dark => ThemeColorPreview {
                background: Color::from_rgb(0.1, 0.1, 0.12),
                foreground: Color::from_rgb(0.9, 0.9, 0.9),
                accent: Color::from_rgb(0.2, 0.6, 1.0),
                primary: Color::from_rgb(0.3, 0.7, 0.9),
                secondary: Color::from_rgb(0.4, 0.8, 0.6),
            },
            ThemeCategory::Light => ThemeColorPreview {
                background: Color::from_rgb(0.98, 0.98, 0.98),
                foreground: Color::from_rgb(0.2, 0.2, 0.2),
                accent: Color::from_rgb(0.0, 0.4, 0.8),
                primary: Color::from_rgb(0.1, 0.5, 0.8),
                secondary: Color::from_rgb(0.2, 0.6, 0.4),
            },
            ThemeCategory::HighContrast => ThemeColorPreview {
                background: Color::from_rgb(0.0, 0.0, 0.0),
                foreground: Color::from_rgb(1.0, 1.0, 1.0),
                accent: Color::from_rgb(1.0, 1.0, 0.0),
                primary: Color::from_rgb(0.0, 1.0, 1.0),
                secondary: Color::from_rgb(1.0, 0.0, 1.0),
            },
            _ => ThemeColorPreview {
                background: Color::from_rgb(0.15, 0.15, 0.2),
                foreground: Color::from_rgb(0.85, 0.85, 0.9),
                accent: Color::from_rgb(0.4, 0.6, 1.0),
                primary: Color::from_rgb(0.5, 0.7, 0.9),
                secondary: Color::from_rgb(0.6, 0.8, 0.7),
            },
        }
    }


    fn format_display_name(&self, name: &str) -> String {
        name.replace("_", " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn set_category_filter(&mut self, category: ThemeCategory) {
        self.category_filter = category;
    }

    pub fn toggle_favorite(&mut self, theme_name: &str) {
        if let Some(theme) = self.available_themes.iter_mut().find(|t| t.name == theme_name) {
            theme.is_favorite = !theme.is_favorite;
            
            if theme.is_favorite {
                self.favorites.push(theme_name.to_string());
            } else {
                self.favorites.retain(|name| name != theme_name);
            }
        }
    }

    pub fn apply_theme(&mut self, theme_name: &str) {
        self.current_theme = theme_name.to_string();
        
        // Add to recently used (remove if exists first)
        self.recently_used.retain(|name| name != theme_name);
        self.recently_used.insert(0, theme_name.to_string());
        
        // Keep only last 10 recent themes
        self.recently_used.truncate(10);
        
        // Update last used timestamp
        if let Some(theme) = self.available_themes.iter_mut().find(|t| t.name == theme_name) {
            theme.last_used = Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
        }
    }

    pub fn set_preview_theme(&mut self, theme_name: Option<String>) {
        self.preview_theme = theme_name;
    }

    pub fn set_background_image(&mut self, path: Option<PathBuf>) {
        self.background_image = path;
    }

    pub fn set_background_opacity(&mut self, opacity: f32) {
        self.background_opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn set_background_blur(&mut self, blur: f32) {
        self.background_blur = blur.max(0.0);
    }

    pub fn toggle_background_settings(&mut self) {
        self.show_background_settings = !self.show_background_settings;
    }

    pub async fn load_custom_theme_async(&mut self) -> Result<(), String> {
        match file_picker::FilePicker::pick_theme_file().await {
            Some(theme_path) => {
                self.load_custom_theme(theme_path)
            }
            None => Err("No theme file selected".to_string())
        }
    }

    pub async fn select_background_image_async(&mut self) -> Result<(), String> {
        match file_picker::FilePicker::pick_background_image().await {
            Some(image_path) => {
                self.set_background_image(Some(image_path));
                Ok(())
            }
            None => Err("No background image selected".to_string())
        }
    }

    pub async fn save_current_theme_async(&self, theme_name: &str) -> Result<(), String> {
        match file_picker::FilePicker::save_theme_file(Some(theme_name)).await {
            Some(save_path) => {
                // Here you would implement the actual theme saving logic
                // For now, we'll just return success
                println!("Theme would be saved to: {:?}", save_path);
                Ok(())
            }
            None => Err("Theme save cancelled".to_string())
        }
    }

    pub fn load_custom_theme(&mut self, theme_path: PathBuf) -> Result<(), String> {
        // This would integrate with the theme loader
        // For now, we'll create a placeholder custom theme
        let theme_name = theme_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("custom_theme")
            .to_string();
        
        let colors = self.generate_sample_colors(&ThemeCategory::Custom);
        
        let custom_theme = ThemeInfo {
            name: theme_name.clone(),
            display_name: self.format_display_name(&theme_name),
            category: ThemeCategory::Custom,
            description: "Custom theme loaded from file".to_string(),
            author: Some("User".to_string()),
            colors,
            is_favorite: false,
            last_used: None,
        };
        
        // Remove existing custom theme with same name
        self.available_themes.retain(|t| t.name != theme_name);
        self.available_themes.push(custom_theme);
        
        // Re-sort themes
        self.available_themes.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        
        Ok(())
    }

    fn get_filtered_themes(&self) -> Vec<&ThemeInfo> {
        self.available_themes
            .iter()
            .filter(|theme| {
                // Category filter
                if self.category_filter != ThemeCategory::All && theme.category != self.category_filter {
                    return false;
                }
                
                // Search filter
                if !self.search_query.is_empty() {
                    let query_lower = self.search_query.to_lowercase();
                    if !theme.display_name.to_lowercase().contains(&query_lower) &&
                       !theme.description.to_lowercase().contains(&query_lower) {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }

    pub fn view<'a, Message: Clone + 'a>(&self, message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        let filtered_themes = self.get_filtered_themes();
        
        let header = self.create_header(message_map);
        let search_bar = self.create_search_bar(message_map);
        let category_filters = self.create_category_filters(message_map);
        let theme_grid = self.create_theme_grid(&filtered_themes, message_map);
        let current_theme_info = self.create_current_theme_info();

        let content = column![
            header,
            Space::with_height(16),
            search_bar,
            Space::with_height(12),
            category_filters,
            Space::with_height(16),
            current_theme_info,
            Space::with_height(16),
            WarpUI::scrollable(theme_grid).height(Length::Fill),
        ]
        .spacing(0)
        .padding(24);

        WarpUI::glass_container(content.into()).into()
    }

    fn create_header<'a, Message: Clone + 'a>(&self, message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        let title_row = row![
            WarpUI::section_header(
                "🎨 Theme Selector",
                Some(&format!("{} themes available", self.available_themes.len()))
            ),
            Space::with_width(Length::Fill),
            row![
                WarpUI::secondary_button(
                    "📁 Load Theme",
                    Some(message_map(ThemeSelectorMessage::LoadCustomTheme))
                ),
                Space::with_width(8),
                WarpUI::secondary_button(
                    "🖼️ Background",
                    Some(message_map(ThemeSelectorMessage::ToggleBackgroundSettings))
                ),
            ].spacing(8)
        ].align_items(Alignment::Center);

        if self.show_background_settings {
            column![
                title_row,
                Space::with_height(16),
                self.create_background_settings(message_map)
            ].into()
        } else {
            title_row.into()
        }
    }

    fn create_background_settings<'a, Message: Clone + 'a>(&self, message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        WarpUI::card(
            column![
                text("Background Settings").size(16).style(Color::from_rgb(0.95, 0.95, 0.95)),
                Space::with_height(12),
                
                // Background image selection
                row![
                    if let Some(image_path) = &self.background_image {
                        text(format!("Image: {}", 
                            image_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")))
                            .size(14)
                            .style(Color::from_rgba(0.8, 0.8, 0.8, 0.9))
                    } else {
                        text("No background image selected")
                            .size(14)
                            .style(Color::from_rgba(0.6, 0.6, 0.6, 0.8))
                    },
                    Space::with_width(Length::Fill),
                    if self.background_image.is_some() {
                        WarpUI::secondary_button(
                            "Remove",
                            Some(message_map(ThemeSelectorMessage::RemoveBackgroundImage))
                        )
                    } else {
                        WarpUI::secondary_button(
                            "📁 Select Image",
                            Some(message_map(ThemeSelectorMessage::SelectBackgroundImage))
                        )
                    }
                ].align_items(Alignment::Center),
                
                Space::with_height(16),
                
                // Opacity control
                row![
                    text("Opacity").size(14).width(Length::Fixed(80.0)),
                    slider(0.0..=1.0, self.background_opacity, move |v| {
                        message_map(ThemeSelectorMessage::BackgroundOpacityChanged(v))
                    })
                    .width(Length::Fixed(200.0)),
                    Space::with_width(12),
                    text(format!("{:.0}%", self.background_opacity * 100.0))
                        .size(14)
                        .width(Length::Fixed(50.0))
                ].align_items(Alignment::Center),
                
                Space::with_height(12),
                
                // Blur control  
                row![
                    text("Blur").size(14).width(Length::Fixed(80.0)),
                    slider(0.0..=10.0, self.background_blur, move |v| {
                        message_map(ThemeSelectorMessage::BackgroundBlurChanged(v))
                    })
                    .width(Length::Fixed(200.0)),
                    Space::with_width(12),
                    text(format!("{:.1}px", self.background_blur))
                        .size(14)
                        .width(Length::Fixed(50.0))
                ].align_items(Alignment::Center)
            ]
            .spacing(8)
            .padding(16)
            .into()
        ).into()
    }

    fn create_search_bar<'a, Message: Clone + 'a>(&self, message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        row![
            text("🔍").size(16),
            Space::with_width(12),
            WarpUI::text_input(
                "Search themes by name or description...",
                &self.search_query,
                move |query| message_map(ThemeSelectorMessage::SearchQueryChanged(query))
            ).width(Length::Fill),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn create_category_filters<'a, Message: Clone + 'a>(&self, message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        let categories = [
            (ThemeCategory::All, "All", "🌈"),
            (ThemeCategory::Dark, "Dark", "🌙"),
            (ThemeCategory::Light, "Light", "☀️"),
            (ThemeCategory::HighContrast, "High Contrast", "⚫"),
            (ThemeCategory::Colorful, "Colorful", "🎨"),
            (ThemeCategory::Minimal, "Minimal", "⚪"),
            (ThemeCategory::Special, "Special", "✨"),
        ];

        let mut filter_row = row![].spacing(8);

        for (category, label, icon) in categories {
            let is_active = self.category_filter == category;
            let button_text = format!("{} {}", icon, label);
            
            let btn = if is_active {
                WarpUI::primary_button(
                    &button_text,
                    Some(message_map(ThemeSelectorMessage::CategoryFilterChanged(category)))
                )
            } else {
                WarpUI::secondary_button(
                    &button_text,
                    Some(message_map(ThemeSelectorMessage::CategoryFilterChanged(category)))
                )
            };

            filter_row = filter_row.push(btn);
        }

        WarpUI::scrollable(filter_row.into()).into()
    }

    fn create_current_theme_info<'a, Message: Clone + 'a>(&self) -> Element<'a, Message> {
        if let Some(current) = self.available_themes.iter().find(|t| t.name == self.current_theme) {
            let color_preview = self.create_color_preview(&current.colors);
            
            let info = column![
                row![
                    text(&current.display_name).size(18).style(Color::from_rgb(0.95, 0.95, 0.95)),
                    Space::with_width(Length::Fill),
                    if current.is_favorite {
                        text("⭐").size(16).style(Color::from_rgb(1.0, 0.8, 0.0))
                    } else {
                        text("☆").size(16).style(Color::from_rgba(0.5, 0.5, 0.5, 0.8))
                    }
                ].align_items(Alignment::Center),
                
                Space::with_height(8),
                
                if !current.description.is_empty() {
                    text(&current.description).size(14).style(Color::from_rgba(0.8, 0.8, 0.8, 0.9))
                } else {
                    text("Currently active theme").size(14).style(Color::from_rgba(0.8, 0.8, 0.8, 0.9))
                },
                
                Space::with_height(12),
                color_preview,
            ].spacing(0);

            WarpUI::card(info.into()).into()
        } else {
            Space::with_height(0).into()
        }
    }

    fn create_theme_grid<'a, Message: Clone + 'a>(&self, themes: &[&ThemeInfo], message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        let mut grid = column![].spacing(12);
        let mut current_row = row![].spacing(12);
        let themes_per_row = 3;

        for (index, theme) in themes.iter().enumerate() {
            let theme_card = self.create_theme_card(theme, message_map);
            current_row = current_row.push(theme_card);

            if (index + 1) % themes_per_row == 0 || index == themes.len() - 1 {
                grid = grid.push(current_row);
                current_row = row![].spacing(12);
            }
        }

        if themes.is_empty() {
            return container(
                column![
                    text("🤔").size(48).style(Color::from_rgba(0.5, 0.5, 0.5, 0.8)),
                    Space::with_height(16),
                    text("No themes found").size(18).style(Color::from_rgba(0.7, 0.7, 0.7, 0.9)),
                    text("Try adjusting your search or category filter").size(14).style(Color::from_rgba(0.6, 0.6, 0.6, 0.8)),
                ]
                .align_items(Alignment::Center)
                .spacing(8)
            )
            .center_x()
            .center_y()
            .height(Length::Fixed(200.0))
            .width(Length::Fill)
            .into();
        }

        grid.into()
    }

    fn create_theme_card<'a, Message: Clone + 'a>(&self, theme: &ThemeInfo, message_map: &'a dyn Fn(ThemeSelectorMessage) -> Message) -> Element<'a, Message> {
        let is_current = theme.name == self.current_theme;
        let is_preview = self.preview_theme.as_ref() == Some(&theme.name);
        
        let color_preview = self.create_color_preview(&theme.colors);
        
        let header = row![
            column![
                text(&theme.display_name).size(16).style(
                    if is_current { 
                        Color::from_rgb(0.0, 0.8, 1.0) 
                    } else { 
                        Color::from_rgb(0.95, 0.95, 0.95) 
                    }
                ),
                if !theme.description.is_empty() {
                    text(&theme.description).size(12).style(Color::from_rgba(0.7, 0.7, 0.7, 0.9))
                } else {
                    text("").size(0)
                }
            ].spacing(4),
            Space::with_width(Length::Fill),
            column![
                button(text(if theme.is_favorite { "⭐" } else { "☆" }).size(14))
                    .on_press(message_map(ThemeSelectorMessage::ToggleFavorite(theme.name.clone())))
                    .style(theme::Button::Custom(Box::new(FavoriteButtonStyle {
                        is_favorite: theme.is_favorite
                    }))),
                Space::with_height(4),
                text(self.get_category_icon(&theme.category)).size(12)
            ].align_items(Alignment::Center)
        ].align_items(Alignment::Start);

        let content = column![
            header,
            Space::with_height(12),
            color_preview,
            Space::with_height(12),
            row![
                if is_current {
                    WarpUI::primary_button("✓ Active", None)
                } else {
                    WarpUI::secondary_button(
                        "Apply",
                        Some(message_map(ThemeSelectorMessage::ApplyTheme(theme.name.clone())))
                    )
                },
                Space::with_width(8),
                WarpUI::secondary_button(
                    if is_preview { "Hide Preview" } else { "Preview" },
                    Some(message_map(ThemeSelectorMessage::PreviewTheme(
                        if is_preview { None } else { Some(theme.name.clone()) }
                    )))
                )
            ]
        ].spacing(0);

        let card_style = if is_current {
            theme::Container::Custom(Box::new(ActiveThemeCardStyle))
        } else if is_preview {
            theme::Container::Custom(Box::new(PreviewThemeCardStyle))
        } else {
            theme::Container::Custom(Box::new(crate::ui::modern_components::WarpModernCard::default()))
        };

        container(content)
            .style(card_style)
            .padding(16)
            .width(Length::Fixed(280.0))
            .into()
    }

    fn create_color_preview<'a, Message: Clone + 'a>(&self, colors: &ThemeColorPreview) -> Element<'a, Message> {
        let color_blocks = row![
            self.create_color_block(colors.background, "BG"),
            self.create_color_block(colors.foreground, "FG"),
            self.create_color_block(colors.accent, "AC"),
            self.create_color_block(colors.primary, "P1"),
            self.create_color_block(colors.secondary, "P2"),
        ].spacing(4);

        container(color_blocks)
            .padding(8)
            .style(theme::Container::Custom(Box::new(ColorPreviewContainer)))
            .into()
    }

    fn create_color_block<'a, Message: Clone + 'a>(&self, color: Color, label: &str) -> Element<'a, Message> {
        container(
            text(label).size(10).style(
                if self.is_light_color(color) {
                    Color::BLACK
                } else {
                    Color::WHITE
                }
            )
        )
        .style(theme::Container::Custom(Box::new(ColorBlockStyle { color })))
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(24.0))
        .center_x()
        .center_y()
        .into()
    }

    fn is_light_color(&self, color: Color) -> bool {
        // Simple luminance calculation
        let luminance = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
        luminance > 0.5
    }

    fn get_category_icon(&self, category: &ThemeCategory) -> &'static str {
        match category {
            ThemeCategory::All => "🌈",
            ThemeCategory::Dark => "🌙",
            ThemeCategory::Light => "☀️",
            ThemeCategory::HighContrast => "⚫",
            ThemeCategory::Colorful => "🎨",
            ThemeCategory::Minimal => "⚪",
            ThemeCategory::Special => "✨",
            ThemeCategory::Custom => "🔧",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ThemeSelectorMessage {
    SearchQueryChanged(String),
    CategoryFilterChanged(ThemeCategory),
    ToggleFavorite(String),
    ApplyTheme(String),
    PreviewTheme(Option<String>),
    LoadCustomTheme,
    SelectBackgroundImage,
    RemoveBackgroundImage,
    BackgroundOpacityChanged(f32),
    BackgroundBlurChanged(f32),
    ToggleBackgroundSettings,
    CustomThemeLoaded(PathBuf),
    BackgroundImageSelected(PathBuf),
}

// Custom styles for theme selector
#[derive(Debug, Clone)]
struct ActiveThemeCardStyle;

impl container::StyleSheet for ActiveThemeCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.5, 1.0, 0.1))),
            border: Border {
                color: Color::from_rgb(0.0, 0.5, 1.0),
                width: 2.0,
                radius: 12.0.into(),
            },
            text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.5, 1.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct PreviewThemeCardStyle;

impl container::StyleSheet for PreviewThemeCardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.8, 0.4, 0.0, 0.1))),
            border: Border {
                color: Color::from_rgb(0.8, 0.4, 0.0),
                width: 2.0,
                radius: 12.0.into(),
            },
            text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
            shadow: Shadow {
                color: Color::from_rgba(0.8, 0.4, 0.0, 0.3),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct FavoriteButtonStyle {
    is_favorite: bool,
}

impl button::StyleSheet for FavoriteButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: if self.is_favorite {
                Color::from_rgb(1.0, 0.8, 0.0)
            } else {
                Color::from_rgba(0.5, 0.5, 0.5, 0.8)
            },
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: if self.is_favorite {
                Color::from_rgb(1.0, 0.9, 0.2)
            } else {
                Color::from_rgba(0.7, 0.7, 0.7, 1.0)
            },
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
            shadow_offset: Vector::new(0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
struct ColorPreviewContainer;

impl container::StyleSheet for ColorPreviewContainer {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.3))),
            border: Border {
                color: Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: None,
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::new(0.0, 0.0),
                blur_radius: 0.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct ColorBlockStyle {
    color: Color,
}

impl container::StyleSheet for ColorBlockStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.color)),
            border: Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: None,
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
        }
    }
}

impl Default for ThemeSelector {
    fn default() -> Self {
        Self::new()
    }
}
