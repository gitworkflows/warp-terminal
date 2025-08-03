#!/bin/bash

# Warp Terminal Configuration Restore Script
# This script checks and restores missing ~/.warp configuration files and directories

set -euo pipefail

WARP_DIR="/Users/$(whoami)/.warp"
BACKUP_DIR="/Users/$(whoami)/.warp/backups"
CONFIG_DIR="/Users/$(whoami)/.config/warp"

echo "🔍 Checking Warp Terminal configuration..."
echo "Current user: $(whoami)"
echo "Warp directory: $WARP_DIR"

# Function to create directory if it doesn't exist
create_dir_if_missing() {
    local dir="$1"
    local description="$2"
    
    if [[ ! -d "$dir" ]]; then
        echo "📁 Creating missing directory: $dir ($description)"
        mkdir -p "$dir"
    else
        echo "✅ Directory exists: $dir"
    fi
}

# Function to create file if it doesn't exist
create_file_if_missing() {
    local file="$1"
    local content="$2"
    local description="$3"
    
    if [[ ! -f "$file" ]]; then
        echo "📄 Creating missing file: $file ($description)"
        echo "$content" > "$file"
    else
        echo "✅ File exists: $file"
    fi
}

echo ""
echo "🔧 Ensuring essential Warp directories exist..."

# Core directories
create_dir_if_missing "$WARP_DIR/blocks" "Terminal blocks storage"
create_dir_if_missing "$WARP_DIR/cache" "Application cache"
create_dir_if_missing "$WARP_DIR/config" "Configuration files"
create_dir_if_missing "$WARP_DIR/custom_commands" "Custom commands"
create_dir_if_missing "$WARP_DIR/launch_configurations" "Launch configurations"
create_dir_if_missing "$WARP_DIR/logs" "Application logs"
create_dir_if_missing "$WARP_DIR/plugins" "Plugin directory"
create_dir_if_missing "$WARP_DIR/scripts" "User scripts"
create_dir_if_missing "$WARP_DIR/terminal_sessions" "Terminal session data"

# Backup directory structure
create_dir_if_missing "$BACKUP_DIR/settings" "Settings backups"
create_dir_if_missing "$BACKUP_DIR/themes" "Theme backups"
create_dir_if_missing "$BACKUP_DIR/workflows" "Workflow backups"

# Alternative config directory (some apps use ~/.config)
create_dir_if_missing "$CONFIG_DIR" "XDG config directory for Warp"

echo ""
echo "🔧 Checking essential configuration files..."

# Default user preferences
USER_PREFS='{
  "appearance": {
    "theme": "dark",
    "font_family": "Hack",
    "font_size": 14,
    "cursor_style": "block"
  },
  "behavior": {
    "auto_suggestions": true,
    "command_palette": true,
    "vim_mode": false
  },
  "terminal": {
    "shell": "/bin/bash",
    "startup_command": null,
    "working_directory": "~/",
    "tab_width": 4
  }
}'

create_file_if_missing "$WARP_DIR/config/user_preferences.json" "$USER_PREFS" "User preferences"

# Application state
APP_STATE='{
  "version": "1.0.0",
  "last_session": null,
  "window_state": {
    "width": 1200,
    "height": 800,
    "maximized": false
  },
  "tabs": [],
  "active_tab": 0
}'

create_file_if_missing "$WARP_DIR/config/app_state.json" "$APP_STATE" "Application state"

# Command history configuration
HISTORY_CONFIG='{
  "max_entries": 10000,
  "dedup_consecutive": true,
  "save_on_exit": true,
  "exclude_patterns": [
    "rm -rf *",
    "sudo rm *",
    "passwd*",
    "ssh-keygen*"
  ],
  "include_timestamps": true
}'

create_file_if_missing "$WARP_DIR/config/history.json" "$HISTORY_CONFIG" "Command history config"

# Keybindings
KEYBINDINGS='{
  "global": {
    "new_tab": "Cmd+T",
    "close_tab": "Cmd+W",
    "next_tab": "Cmd+]",
    "prev_tab": "Cmd+[",
    "split_pane_horizontal": "Cmd+D",
    "split_pane_vertical": "Cmd+Shift+D"
  },
  "terminal": {
    "clear_screen": "Cmd+K",
    "copy": "Cmd+C",
    "paste": "Cmd+V",
    "search": "Cmd+F"
  }
}'

create_file_if_missing "$WARP_DIR/config/keybindings.json" "$KEYBINDINGS" "Keybindings"

echo ""
echo "🎨 Checking theme files..."

# Ensure default themes exist
if [[ ! -f "$WARP_DIR/themes/warp_bundled/dark.yaml" ]]; then
    echo "📄 Creating default dark theme..."
    create_dir_if_missing "$WARP_DIR/themes/warp_bundled" "Default themes"
    
    DARK_THEME='name: "Dark"
author: "Warp Team"
accent: "#00D4FF"
background: "#0D1117"
foreground: "#E6EDF3"
details: "darker"

terminal_colors:
  normal:
    black: "#21262D"
    red: "#F85149"
    green: "#7EE787"
    yellow: "#F9E2AF"
    blue: "#79C0FF"
    magenta: "#D2A8FF"
    cyan: "#39D0D8"
    white: "#E6EDF3"
  bright:
    black: "#8B949E"
    red: "#FF7B72"
    green: "#56D364"
    yellow: "#E3B341"
    blue: "#58A6FF"
    magenta: "#BC8CFF"
    cyan: "#39D0D8"
    white: "#F0F6FC"'
    
    echo "$DARK_THEME" > "$WARP_DIR/themes/warp_bundled/dark.yaml"
fi

if [[ ! -f "$WARP_DIR/themes/warp_bundled/light.yaml" ]]; then
    echo "📄 Creating default light theme..."
    
    LIGHT_THEME='name: "Light"
author: "Warp Team"
accent: "#0969DA"
background: "#FFFFFF"
foreground: "#24292F"
details: "lighter"

terminal_colors:
  normal:
    black: "#24292F"
    red: "#CF222E"
    green: "#116329"
    yellow: "#4D2D00"
    blue: "#0969DA"
    magenta: "#8250DF"
    cyan: "#1B7C83"
    white: "#6E7781"
  bright:
    black: "#656D76"
    red: "#A40E26"
    green: "#1A7F37"
    yellow: "#633C01"
    blue: "#218BFF"
    magenta: "#A475F9"
    cyan: "#3192AA"
    white: "#8C959F"'
    
    echo "$LIGHT_THEME" > "$WARP_DIR/themes/warp_bundled/light.yaml"
fi

echo ""
echo "🚀 Creating launch configurations..."

# Default launch configurations
BASH_CONFIG='{
  "name": "Bash",
  "command": "/bin/bash",
  "args": ["--login"],
  "env": {},
  "working_directory": "~"
}'

create_file_if_missing "$WARP_DIR/launch_configurations/bash.json" "$BASH_CONFIG" "Bash launch config"

ZSH_CONFIG='{
  "name": "Zsh",
  "command": "/bin/zsh",
  "args": ["--login"],
  "env": {},
  "working_directory": "~"
}'

create_file_if_missing "$WARP_DIR/launch_configurations/zsh.json" "$ZSH_CONFIG" "Zsh launch config"

echo ""
echo "🔧 Setting up logging..."

# Create log files
touch "$WARP_DIR/logs/application.log"
touch "$WARP_DIR/logs/terminal.log"
touch "$WARP_DIR/logs/errors.log"

# Log configuration
LOG_CONFIG='{
  "level": "info",
  "file_logging": true,
  "console_logging": true,
  "max_file_size": "10MB",
  "max_files": 5,
  "log_directory": "'$WARP_DIR'/logs"
}'

create_file_if_missing "$WARP_DIR/config/logging.json" "$LOG_CONFIG" "Logging configuration"

echo ""
echo "🛠️ Checking permissions..."

# Set proper permissions
chmod -R 755 "$WARP_DIR"
chmod 644 "$WARP_DIR"/config/*.json 2>/dev/null || true
chmod 644 "$WARP_DIR"/logs/*.log 2>/dev/null || true

echo ""
echo "📊 Configuration Summary:"
echo "========================"
echo "✅ Core directories: $(find "$WARP_DIR" -type d | wc -l | tr -d ' ') created/verified"
echo "✅ Configuration files: $(find "$WARP_DIR/config" -name "*.json" 2>/dev/null | wc -l | tr -d ' ') created/verified"
echo "✅ Theme files: $(find "$WARP_DIR/themes" -name "*.yaml" 2>/dev/null | wc -l | tr -d ' ') created/verified"
echo "✅ Launch configurations: $(find "$WARP_DIR/launch_configurations" -name "*.json" 2>/dev/null | wc -l | tr -d ' ') created/verified"
echo "✅ Log files: $(find "$WARP_DIR/logs" -name "*.log" 2>/dev/null | wc -l | tr -d ' ') created/verified"

echo ""
echo "🎉 Warp configuration restore completed!"
echo ""
echo "📝 Next steps:"
echo "1. Build the terminal: cargo build --release"
echo "2. Run the terminal: ./target/release/warp-terminal"
echo "3. Or use the preview app: open /Volumes/Warp/WarpPreview.app"
echo ""
echo "🔍 If you encounter issues, check the logs in: $WARP_DIR/logs/"
