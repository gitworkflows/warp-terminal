//! Theme preview generation
//!
//! This module provides advanced preview generation capabilities including:
//! - SVG preview generation with customizable templates
//! - HTML preview pages
//! - Terminal screenshots
//! - Color palette exports

use crate::{Theme, Result, ThemeError};
use std::collections::HashMap;
use std::path::Path;

/// Preview formats supported
#[derive(Debug, Clone)]
pub enum PreviewFormat {
    Svg,
    Html,
    Png,
    Json,
}

/// Preview generation options
#[derive(Debug, Clone)]
pub struct PreviewOptions {
    pub format: PreviewFormat,
    pub width: u32,
    pub height: u32,
    pub show_terminal_colors: bool,
    pub show_code_sample: bool,
    pub template_name: Option<String>,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            format: PreviewFormat::Svg,
            width: 400,
            height: 300,
            show_terminal_colors: true,
            show_code_sample: true,
            template_name: None,
        }
    }
}

/// Theme preview generator
pub struct PreviewGenerator;

impl PreviewGenerator {
    /// Generate preview for a theme
    pub fn generate_preview(theme: &Theme, options: &PreviewOptions) -> Result<String> {
        match options.format {
            PreviewFormat::Svg => Self::generate_svg_preview(theme, options),
            PreviewFormat::Html => Self::generate_html_preview(theme, options),
            PreviewFormat::Png => Self::generate_png_preview(theme, options),
            PreviewFormat::Json => Self::generate_json_preview(theme, options),
        }
    }

    /// Generate SVG preview
    fn generate_svg_preview(theme: &Theme, options: &PreviewOptions) -> Result<String> {
        let template = Self::get_svg_template(options.template_name.as_deref())?;
        let mut context = Self::create_template_context(theme);

        // Add dimension context
        context.insert("width".to_string(), options.width.to_string());
        context.insert("height".to_string(), options.height.to_string());

        // Generate terminal colors section if requested
        if options.show_terminal_colors {
            context.insert("terminal_colors_section".to_string(), 
                Self::generate_terminal_colors_svg(theme));
        }

        // Generate code sample section if requested
        if options.show_code_sample {
            context.insert("code_sample_section".to_string(), 
                Self::generate_code_sample_svg(theme));
        }

        Self::render_template(&template, &context)
    }

    /// Generate HTML preview
    fn generate_html_preview(theme: &Theme, _options: &PreviewOptions) -> Result<String> {
        let template = Self::get_html_template()?;
        let mut context = Self::create_template_context(theme);

        // Add CSS variables for theme colors
        context.insert("css_variables".to_string(), Self::generate_css_variables(theme));
        
        // Add terminal colors as JSON for JavaScript
        context.insert("terminal_colors_json".to_string(), 
            Self::generate_terminal_colors_json(theme)?);

        Self::render_template(&template, &context)
    }

    /// Generate PNG preview (placeholder - would require image generation library)
    fn generate_png_preview(_theme: &Theme, _options: &PreviewOptions) -> Result<String> {
        Err(ThemeError::InvalidFormat("PNG generation not yet implemented".to_string()))
    }

    /// Generate JSON preview data
    fn generate_json_preview(theme: &Theme, _options: &PreviewOptions) -> Result<String> {
        let mut preview_data = HashMap::new();
        
        preview_data.insert("name", serde_json::Value::String(theme.display_name()));
        preview_data.insert("accent", serde_json::Value::String(theme.accent.to_hex()));
        preview_data.insert("background", serde_json::Value::String(theme.background.to_hex()));
        preview_data.insert("foreground", serde_json::Value::String(theme.foreground.to_hex()));
        preview_data.insert("is_dark", serde_json::Value::Bool(theme.is_dark()));

        // Add terminal colors
        let normal_colors = &theme.terminal_colors.normal;
        let bright_colors = &theme.terminal_colors.bright;

        let terminal_colors = serde_json::json!({
            "normal": {
                "black": normal_colors.black.to_hex(),
                "red": normal_colors.red.to_hex(),
                "green": normal_colors.green.to_hex(),
                "yellow": normal_colors.yellow.to_hex(),
                "blue": normal_colors.blue.to_hex(),
                "magenta": normal_colors.magenta.to_hex(),
                "cyan": normal_colors.cyan.to_hex(),
                "white": normal_colors.white.to_hex(),
            },
            "bright": {
                "black": bright_colors.black.to_hex(),
                "red": bright_colors.red.to_hex(),
                "green": bright_colors.green.to_hex(),
                "yellow": bright_colors.yellow.to_hex(),
                "blue": bright_colors.blue.to_hex(),
                "magenta": bright_colors.magenta.to_hex(),
                "cyan": bright_colors.cyan.to_hex(),
                "white": bright_colors.white.to_hex(),
            }
        });

        preview_data.insert("terminal_colors", terminal_colors);

        serde_json::to_string_pretty(&preview_data).map_err(|e| {
            ThemeError::InvalidFormat(format!("Failed to serialize JSON: {}", e))
        })
    }

    /// Create template context from theme
    fn create_template_context(theme: &Theme) -> HashMap<String, String> {
        let mut context = HashMap::new();
        
        context.insert("name".to_string(), theme.display_name());
        context.insert("accent".to_string(), theme.accent.to_hex());
        context.insert("background".to_string(), theme.background.to_hex());
        context.insert("foreground".to_string(), theme.foreground.to_hex());
        
        if let Some(cursor) = theme.cursor {
            context.insert("cursor".to_string(), cursor.to_hex());
        } else {
            context.insert("cursor".to_string(), theme.accent.to_hex());
        }

        // Terminal colors
        let normal = &theme.terminal_colors.normal;
        let bright = &theme.terminal_colors.bright;

        context.insert("black".to_string(), normal.black.to_hex());
        context.insert("red".to_string(), normal.red.to_hex());
        context.insert("green".to_string(), normal.green.to_hex());
        context.insert("yellow".to_string(), normal.yellow.to_hex());
        context.insert("blue".to_string(), normal.blue.to_hex());
        context.insert("magenta".to_string(), normal.magenta.to_hex());
        context.insert("cyan".to_string(), normal.cyan.to_hex());
        context.insert("white".to_string(), normal.white.to_hex());

        context.insert("brblack".to_string(), bright.black.to_hex());
        context.insert("brred".to_string(), bright.red.to_hex());
        context.insert("brgreen".to_string(), bright.green.to_hex());
        context.insert("bryellow".to_string(), bright.yellow.to_hex());
        context.insert("brblue".to_string(), bright.blue.to_hex());
        context.insert("brmagenta".to_string(), bright.magenta.to_hex());
        context.insert("brcyan".to_string(), bright.cyan.to_hex());
        context.insert("brwhite".to_string(), bright.white.to_hex());

        context
    }

    /// Get SVG template
    fn get_svg_template(template_name: Option<&str>) -> Result<String> {
        match template_name {
            Some("compact") => Ok(Self::get_compact_svg_template()),
            Some("detailed") => Ok(Self::get_detailed_svg_template()),
            Some("terminal") => Ok(Self::get_terminal_svg_template()),
            _ => Ok(Self::get_default_svg_template()),
        }
    }

    /// Default SVG template
    fn get_default_svg_template() -> String {
        r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <style>
      .theme-bg { fill: {background}; }
      .theme-fg { fill: {foreground}; }
      .theme-accent { fill: {accent}; }
      .terminal-text { font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace; font-size: 12px; }
      .theme-name { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; font-size: 16px; font-weight: bold; }
      .color-swatch { stroke: {foreground}; stroke-width: 1; stroke-opacity: 0.3; }
    </style>
  </defs>
  
  <!-- Background -->
  <rect width="100%" height="100%" class="theme-bg" rx="8"/>
  
  <!-- Theme name -->
  <text x="20" y="30" class="theme-name theme-fg">{name}</text>
  
  <!-- Terminal window mockup -->
  <rect x="20" y="50" width="360" height="200" fill="{background}" stroke="{foreground}" stroke-width="2" stroke-opacity="0.2" rx="6"/>
  
  <!-- Terminal content -->
  <text x="30" y="75" class="terminal-text" fill="{green}">$ ls -la</text>
  <text x="30" y="95" class="terminal-text" fill="{blue}">drwxr-xr-x</text>
  <text x="120" y="95" class="terminal-text" fill="{cyan}">user</text>
  <text x="160" y="95" class="terminal-text" fill="{yellow}">group</text>
  <text x="210" y="95" class="terminal-text" fill="{magenta}">4096</text>
  <text x="250" y="95" class="terminal-text" fill="{white}">file.txt</text>
  
  <text x="30" y="115" class="terminal-text" fill="{green}">$ git status</text>
  <text x="30" y="135" class="terminal-text" fill="{red}">modified: </text>
  <text x="100" y="135" class="terminal-text" fill="{white}">src/main.rs</text>
  <text x="30" y="155" class="terminal-text" fill="{green}">new file: </text>
  <text x="100" y="155" class="terminal-text" fill="{white}">README.md</text>
  
  <!-- Cursor -->
  <rect x="30" y="175" width="8" height="15" fill="{accent}" opacity="0.8"/>
  
  {terminal_colors_section}
  {code_sample_section}
</svg>"#.to_string()
    }

    /// Compact SVG template
    fn get_compact_svg_template() -> String {
        r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <style>
      .theme-bg { fill: {background}; }
      .theme-fg { fill: {foreground}; }
      .theme-accent { fill: {accent}; }
      .theme-name { font-family: -apple-system, sans-serif; font-size: 14px; font-weight: 600; }
      .color-swatch { stroke: {foreground}; stroke-width: 1; stroke-opacity: 0.2; }
    </style>
  </defs>
  
  <!-- Background -->
  <rect width="100%" height="100%" class="theme-bg" rx="4"/>
  
  <!-- Theme name -->
  <text x="10" y="20" class="theme-name theme-fg">{name}</text>
  
  <!-- Color swatches -->
  <rect x="10" y="30" width="20" height="20" fill="{red}" class="color-swatch" rx="2"/>
  <rect x="35" y="30" width="20" height="20" fill="{green}" class="color-swatch" rx="2"/>
  <rect x="60" y="30" width="20" height="20" fill="{yellow}" class="color-swatch" rx="2"/>
  <rect x="85" y="30" width="20" height="20" fill="{blue}" class="color-swatch" rx="2"/>
  <rect x="110" y="30" width="20" height="20" fill="{magenta}" class="color-swatch" rx="2"/>
  <rect x="135" y="30" width="20" height="20" fill="{cyan}" class="color-swatch" rx="2"/>
  <rect x="160" y="30" width="20" height="20" fill="{accent}" class="color-swatch" rx="2"/>
</svg>"#.to_string()
    }

    /// Detailed SVG template
    fn get_detailed_svg_template() -> String {
        r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <style>
      .theme-bg { fill: {background}; }
      .theme-fg { fill: {foreground}; }
      .theme-accent { fill: {accent}; }
      .terminal-text { font-family: 'SF Mono', 'Monaco', monospace; font-size: 11px; }
      .theme-name { font-family: -apple-system, sans-serif; font-size: 18px; font-weight: bold; }
      .section-title { font-family: -apple-system, sans-serif; font-size: 12px; font-weight: 600; }
      .color-label { font-family: -apple-system, sans-serif; font-size: 10px; }
      .color-swatch { stroke: {foreground}; stroke-width: 1; stroke-opacity: 0.3; }
    </style>
  </defs>
  
  <!-- Background -->
  <rect width="100%" height="100%" class="theme-bg" rx="12"/>
  
  <!-- Theme name and type -->
  <text x="20" y="35" class="theme-name theme-fg">{name}</text>
  <text x="20" y="55" class="section-title theme-accent">Terminal Theme Preview</text>
  
  <!-- Main colors section -->
  <text x="20" y="85" class="section-title theme-fg">Main Colors</text>
  <rect x="20" y="95" width="30" height="20" fill="{background}" class="color-swatch" rx="3"/>
  <text x="20" y="125" class="color-label theme-fg">Background</text>
  
  <rect x="70" y="95" width="30" height="20" fill="{foreground}" class="color-swatch" rx="3"/>
  <text x="70" y="125" class="color-label theme-fg">Foreground</text>
  
  <rect x="120" y="95" width="30" height="20" fill="{accent}" class="color-swatch" rx="3"/>
  <text x="120" y="125" class="color-label theme-fg">Accent</text>
  
  <!-- Terminal colors section -->
  <text x="200" y="85" class="section-title theme-fg">Terminal Colors</text>
  <rect x="200" y="95" width="15" height="15" fill="{red}" class="color-swatch" rx="2"/>
  <rect x="220" y="95" width="15" height="15" fill="{green}" class="color-swatch" rx="2"/>
  <rect x="240" y="95" width="15" height="15" fill="{yellow}" class="color-swatch" rx="2"/>
  <rect x="260" y="95" width="15" height="15" fill="{blue}" class="color-swatch" rx="2"/>
  <rect x="280" y="95" width="15" height="15" fill="{magenta}" class="color-swatch" rx="2"/>
  <rect x="300" y="95" width="15" height="15" fill="{cyan}" class="color-swatch" rx="2"/>
  
  <!-- Code sample -->
  <text x="20" y="160" class="section-title theme-fg">Code Sample</text>
  <rect x="20" y="170" width="360" height="120" fill="{background}" stroke="{foreground}" stroke-width="1" stroke-opacity="0.2" rx="6"/>
  
  <text x="30" y="190" class="terminal-text" fill="{blue}">fn</text>
  <text x="50" y="190" class="terminal-text" fill="{yellow}"> main</text>
  <text x="80" y="190" class="terminal-text" fill="{white}">()</text>
  <text x="100" y="190" class="terminal-text" fill="{white}"> {</text>
  
  <text x="40" y="210" class="terminal-text" fill="{blue}">let</text>
  <text x="65" y="210" class="terminal-text" fill="{white}"> message = </text>
  <text x="140" y="210" class="terminal-text" fill="{green}">"Hello, World!"</text>
  <text x="240" y="210" class="terminal-text" fill="{white}">;</text>
  
  <text x="40" y="230" class="terminal-text" fill="{magenta}">println!</text>
  <text x="85" y="230" class="terminal-text" fill="{white}">(</text>
  <text x="95" y="230" class="terminal-text" fill="{green}">"{}"</text>
  <text x="115" y="230" class="terminal-text" fill="{white}">, message);</text>
  
  <text x="30" y="250" class="terminal-text" fill="{white}">}</text>
  
  <!-- Output section -->
  <text x="30" y="275" class="terminal-text" fill="{green}">Hello, World!</text>
</svg>"#.to_string()
    }

    /// Terminal-focused SVG template
    fn get_terminal_svg_template() -> String {
        r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <style>
      .theme-bg { fill: {background}; }
      .theme-fg { fill: {foreground}; }
      .terminal-text { font-family: 'SF Mono', 'Monaco', monospace; font-size: 13px; }
      .prompt { fill: {accent}; }
    </style>
  </defs>
  
  <!-- Terminal background -->
  <rect width="100%" height="100%" class="theme-bg" rx="8"/>
  
  <!-- Terminal header -->
  <rect x="0" y="0" width="100%" height="30" fill="{foreground}" opacity="0.1" rx="8"/>
  <circle cx="15" cy="15" r="6" fill="{red}" opacity="0.8"/>
  <circle cx="35" cy="15" r="6" fill="{yellow}" opacity="0.8"/>
  <circle cx="55" cy="15" r="6" fill="{green}" opacity="0.8"/>
  
  <!-- Terminal session -->
  <text x="15" y="55" class="terminal-text prompt">❯</text>
  <text x="35" y="55" class="terminal-text" fill="{white}">ls -la</text>
  
  <text x="15" y="75" class="terminal-text" fill="{blue}">drwxr-xr-x</text>
  <text x="90" y="75" class="terminal-text" fill="{cyan}">user</text>
  <text x="130" y="75" class="terminal-text" fill="{yellow}">staff</text>
  <text x="170" y="75" class="terminal-text" fill="{white}">README.md</text>
  
  <text x="15" y="95" class="terminal-text" fill="{blue}">-rw-r--r--</text>
  <text x="90" y="95" class="terminal-text" fill="{cyan}">user</text>
  <text x="130" y="95" class="terminal-text" fill="{yellow}">staff</text>
  <text x="170" y="95" class="terminal-text" fill="{white}">src/</text>
  
  <text x="15" y="115" class="terminal-text prompt">❯</text>
  <text x="35" y="115" class="terminal-text" fill="{white}">git status</text>
  
  <text x="15" y="135" class="terminal-text" fill="{green}">On branch</text>
  <text x="80" y="135" class="terminal-text" fill="{accent}">main</text>
  
  <text x="15" y="155" class="terminal-text" fill="{green}">Changes to be committed:</text>
  <text x="15" y="175" class="terminal-text" fill="{green}">  modified:</text>
  <text x="90" y="175" class="terminal-text" fill="{white}">src/main.rs</text>
  
  <text x="15" y="195" class="terminal-text" fill="{red}">Changes not staged:</text>
  <text x="15" y="215" class="terminal-text" fill="{red}">  modified:</text>
  <text x="90" y="215" class="terminal-text" fill="{white}">Cargo.toml</text>
  
  <text x="15" y="235" class="terminal-text prompt">❯</text>
  <rect x="35" y="225" width="10" height="15" fill="{accent}" opacity="0.7"/>
</svg>"#.to_string()
    }

    /// Get HTML template
    fn get_html_template() -> Result<String> {
        Ok(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} - Theme Preview</title>
    <style>
        :root {
{css_variables}
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: var(--bg-color);
            color: var(--fg-color);
            min-height: 100vh;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            text-align: center;
            margin-bottom: 40px;
        }
        
        .theme-title {
            font-size: 2.5rem;
            font-weight: bold;
            margin: 0;
            color: var(--accent-color);
        }
        
        .theme-subtitle {
            font-size: 1.2rem;
            margin: 10px 0 0 0;
            opacity: 0.8;
        }
        
        .preview-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin-bottom: 40px;
        }
        
        .terminal-window {
            background: var(--bg-color);
            border: 2px solid var(--fg-color);
            border-opacity: 0.2;
            border-radius: 12px;
            overflow: hidden;
            box-shadow: 0 4px 20px rgba(0,0,0,0.3);
        }
        
        .terminal-header {
            background: rgba(255,255,255,0.05);
            padding: 12px 16px;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        
        .terminal-dot {
            width: 12px;
            height: 12px;
            border-radius: 50%;
        }
        
        .dot-red { background: var(--red-color); }
        .dot-yellow { background: var(--yellow-color); }
        .dot-green { background: var(--green-color); }
        
        .terminal-content {
            padding: 20px;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
            font-size: 14px;
            line-height: 1.6;
        }
        
        .prompt { color: var(--accent-color); }
        .command { color: var(--fg-color); }
        .output { color: var(--cyan-color); }
        .error { color: var(--red-color); }
        .success { color: var(--green-color); }
        .warning { color: var(--yellow-color); }
        
        .color-palette {
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 10px;
            margin-bottom: 30px;
        }
        
        .color-swatch {
            aspect-ratio: 1;
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 12px;
            font-weight: bold;
            color: white;
            text-shadow: 0 1px 2px rgba(0,0,0,0.5);
            border: 2px solid rgba(255,255,255,0.1);
        }
        
        .cursor {
            display: inline-block;
            width: 10px;
            height: 18px;
            background: var(--accent-color);
            animation: blink 1s infinite;
        }
        
        @keyframes blink {
            0%, 50% { opacity: 1; }
            51%, 100% { opacity: 0; }
        }
        
        .code-sample {
            background: rgba(0,0,0,0.2);
            padding: 20px;
            border-radius: 8px;
            font-family: 'SF Mono', Monaco, monospace;
            font-size: 14px;
            line-height: 1.6;
            overflow-x: auto;
        }
        
        .keyword { color: var(--blue-color); }
        .string { color: var(--green-color); }
        .comment { color: var(--white-color); opacity: 0.6; }
        .function { color: var(--yellow-color); }
        .type { color: var(--magenta-color); }
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <h1 class="theme-title">{name}</h1>
            <p class="theme-subtitle">Terminal Theme Preview</p>
        </header>
        
        <div class="color-palette">
            <div class="color-swatch" style="background: var(--red-color)">Red</div>
            <div class="color-swatch" style="background: var(--green-color)">Green</div>
            <div class="color-swatch" style="background: var(--yellow-color)">Yellow</div>
            <div class="color-swatch" style="background: var(--blue-color)">Blue</div>
            <div class="color-swatch" style="background: var(--magenta-color)">Magenta</div>
            <div class="color-swatch" style="background: var(--cyan-color)">Cyan</div>
            <div class="color-swatch" style="background: var(--accent-color)">Accent</div>
            <div class="color-swatch" style="background: var(--white-color)">White</div>
        </div>
        
        <div class="preview-grid">
            <div class="terminal-window">
                <div class="terminal-header">
                    <div class="terminal-dot dot-red"></div>
                    <div class="terminal-dot dot-yellow"></div>
                    <div class="terminal-dot dot-green"></div>
                </div>
                <div class="terminal-content">
                    <div><span class="prompt">❯</span> <span class="command">ls -la</span></div>
                    <div class="output">drwxr-xr-x user staff 4096 README.md</div>
                    <div class="output">-rw-r--r-- user staff 2048 src/</div>
                    <div><span class="prompt">❯</span> <span class="command">git status</span></div>
                    <div class="success">On branch main</div>
                    <div class="success">Changes to be committed:</div>
                    <div class="success">  modified: src/main.rs</div>
                    <div class="error">Changes not staged:</div>
                    <div class="error">  modified: Cargo.toml</div>
                    <div><span class="prompt">❯</span> <span class="cursor"></span></div>
                </div>
            </div>
            
            <div class="code-sample">
                <div><span class="keyword">use</span> std::collections::HashMap;</div>
                <div></div>
                <div><span class="keyword">fn</span> <span class="function">main</span>() {</div>
                <div>    <span class="keyword">let</span> <span class="keyword">mut</span> map = HashMap::<span class="type">&lt;String, i32&gt;</span>::new();</div>
                <div>    map.insert(<span class="string">"hello"</span>.to_string(), <span class="number">42</span>);</div>
                <div>    </div>
                <div>    <span class="comment">// Print the result</span></div>
                <div>    println!(<span class="string">"{:?}"</span>, map);</div>
                <div>}</div>
            </div>
        </div>
    </div>
    
    <script>
        const terminalColors = {terminal_colors_json};
        console.log('Theme colors:', terminalColors);
    </script>
</body>
</html>"#.to_string())
    }

    /// Generate terminal colors section for SVG
    fn generate_terminal_colors_svg(theme: &Theme) -> String {
        let normal = &theme.terminal_colors.normal;
        let mut svg = String::new();
        
        svg.push_str(&format!(r#"
  <!-- Terminal color palette -->
  <text x="20" y="300" class="section-title theme-fg">Terminal Colors</text>
  <rect x="20" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="45" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="70" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="95" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="120" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="145" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="170" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
  <rect x="195" y="310" width="20" height="20" fill="{}" class="color-swatch" rx="2"/>
"#, 
            normal.black.to_hex(), normal.red.to_hex(), normal.green.to_hex(), 
            normal.yellow.to_hex(), normal.blue.to_hex(), normal.magenta.to_hex(), 
            normal.cyan.to_hex(), normal.white.to_hex()
        ));
        
        svg
    }

    /// Generate code sample section for SVG
    fn generate_code_sample_svg(_theme: &Theme) -> String {
        // This could be expanded to show more realistic code samples
        String::new()
    }

    /// Generate CSS variables for HTML template
    fn generate_css_variables(theme: &Theme) -> String {
        let normal = &theme.terminal_colors.normal;
        format!(
            "            --bg-color: {};\n\
             --fg-color: {};\n\
             --accent-color: {};\n\
             --red-color: {};\n\
             --green-color: {};\n\
             --yellow-color: {};\n\
             --blue-color: {};\n\
             --magenta-color: {};\n\
             --cyan-color: {};\n\
             --white-color: {};",
            theme.background.to_hex(),
            theme.foreground.to_hex(),
            theme.accent.to_hex(),
            normal.red.to_hex(),
            normal.green.to_hex(),
            normal.yellow.to_hex(),
            normal.blue.to_hex(),
            normal.magenta.to_hex(),
            normal.cyan.to_hex(),
            normal.white.to_hex()
        )
    }

    /// Generate terminal colors JSON for HTML template
    fn generate_terminal_colors_json(theme: &Theme) -> Result<String> {
        let colors = serde_json::json!({
            "normal": {
                "black": theme.terminal_colors.normal.black.to_hex(),
                "red": theme.terminal_colors.normal.red.to_hex(),
                "green": theme.terminal_colors.normal.green.to_hex(),
                "yellow": theme.terminal_colors.normal.yellow.to_hex(),
                "blue": theme.terminal_colors.normal.blue.to_hex(),
                "magenta": theme.terminal_colors.normal.magenta.to_hex(),
                "cyan": theme.terminal_colors.normal.cyan.to_hex(),
                "white": theme.terminal_colors.normal.white.to_hex(),
            },
            "bright": {
                "black": theme.terminal_colors.bright.black.to_hex(),
                "red": theme.terminal_colors.bright.red.to_hex(),
                "green": theme.terminal_colors.bright.green.to_hex(),
                "yellow": theme.terminal_colors.bright.yellow.to_hex(),
                "blue": theme.terminal_colors.bright.blue.to_hex(),
                "magenta": theme.terminal_colors.bright.magenta.to_hex(),
                "cyan": theme.terminal_colors.bright.cyan.to_hex(),
                "white": theme.terminal_colors.bright.white.to_hex(),
            }
        });
        
        serde_json::to_string(&colors).map_err(|e| {
            ThemeError::InvalidFormat(format!("Failed to serialize JSON: {}", e))
        })
    }

    /// Simple template rendering (replace placeholders with values)
    fn render_template(template: &str, context: &HashMap<String, String>) -> Result<String> {
        let mut result = template.to_string();
        
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }

    /// Generate multiple previews for a theme
    pub fn generate_all_previews(theme: &Theme, output_dir: &Path) -> Result<()> {
        let formats = vec![
            (PreviewFormat::Svg, "svg"),
            (PreviewFormat::Html, "html"),
            (PreviewFormat::Json, "json"),
        ];

        for (format, extension) in formats {
            let options = PreviewOptions {
                format,
                ..Default::default()
            };

            let preview = Self::generate_preview(theme, &options)?;
            let filename = format!("{}.{}", theme.display_name().to_lowercase().replace(' ', "_"), extension);
            let output_path = output_dir.join(filename);

            std::fs::write(output_path, preview)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, TerminalColors};

    #[test]
    fn test_svg_preview_generation() {
        let theme = Theme::new(
            Color::rgb(255, 100, 100),
            Color::rgb(30, 30, 30),
            Color::rgb(240, 240, 240),
            TerminalColors::default_dark(),
        );

        let options = PreviewOptions::default();
        let preview = PreviewGenerator::generate_preview(&theme, &options).unwrap();
        
        assert!(preview.contains("<svg"));
        assert!(preview.contains("</svg>"));
        assert!(preview.contains("#ff6464")); // accent color
    }

    #[test]
    fn test_json_preview_generation() {
        let theme = Theme::new(
            Color::rgb(255, 100, 100),
            Color::rgb(30, 30, 30),
            Color::rgb(240, 240, 240),
            TerminalColors::default_dark(),
        );

        let options = PreviewOptions {
            format: PreviewFormat::Json,
            ..Default::default()
        };

        let preview = PreviewGenerator::generate_preview(&theme, &options).unwrap();
        
        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&preview).unwrap();
        assert!(preview.contains("terminal_colors"));
    }
}
