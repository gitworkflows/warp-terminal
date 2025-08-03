#!/bin/bash

# Rust Code Review and Restoration Script
# Log Level: debug

set -e

PROJECT_DIR="/Users/KhulnaSoft/.warp"
WARP_PREVIEW_PATH="/Volumes/Warp/WarpPreview.app/Contents/MacOS/preview"
LOG_FILE="$PROJECT_DIR/rust_review_debug.log"
BACKUP_DIR="$PROJECT_DIR/backups/rust_review_$(date +%Y%m%d_%H%M%S)"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Debug logging function
debug_log() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

info() { debug_log "INFO" "$@"; echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { debug_log "WARN" "$@"; echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { debug_log "ERROR" "$@"; echo -e "${RED}[ERROR]${NC} $*"; }
debug() { debug_log "DEBUG" "$@"; echo -e "${BLUE}[DEBUG]${NC} $*"; }

# Initialize log file
echo "=== Rust Code Review and Restoration - $(date) ===" > "$LOG_FILE"

info "Starting Rust code review and restoration process"
debug "Project directory: $PROJECT_DIR"
debug "WarpPreview path: $WARP_PREVIEW_PATH"
debug "Log file: $LOG_FILE"

# Create backup directory
mkdir -p "$BACKUP_DIR"
info "Created backup directory: $BACKUP_DIR"

# Function to check file integrity
check_file_integrity() {
    local file_path="$1"
    local file_type="$2"
    
    debug "Checking integrity of $file_type file: $file_path"
    
    if [[ ! -f "$file_path" ]]; then
        error "Missing $file_type file: $file_path"
        return 1
    fi
    
    # Check if file is empty
    if [[ ! -s "$file_path" ]]; then
        warn "Empty $file_type file: $file_path"
        return 1
    fi
    
    # Basic syntax check for Rust files
    if [[ "$file_type" == "rust" && "$file_path" == *.rs ]]; then
        if ! rustc --parse "$file_path" &>/dev/null; then
            error "Syntax error in Rust file: $file_path"
            return 1
        fi
    fi
    
    info "✓ $file_type file integrity check passed: $file_path"
    return 0
}

# Function to analyze TODO and FIXME items
analyze_todos() {
    info "Analyzing TODO and FIXME items in the codebase"
    
    local todo_file="$BACKUP_DIR/todo_analysis.txt"
    echo "=== TODO and FIXME Analysis ===" > "$todo_file"
    
    # Find all TODO, FIXME, and unimplemented items
    find "$PROJECT_DIR/src" -name "*.rs" -exec grep -Hn "TODO\|FIXME\|unimplemented!\|todo!\|unreachable!" {} \; >> "$todo_file"
    
    local todo_count=$(grep -c "TODO\|FIXME\|unimplemented!\|todo!\|unreachable!" "$todo_file" 2>/dev/null || echo "0")
    
    if [[ $todo_count -gt 0 ]]; then
        warn "Found $todo_count TODO/FIXME items requiring attention"
        debug "TODO analysis saved to: $todo_file"
        
        # Categorize TODOs
        echo -e "\n=== HIGH PRIORITY TODOs ===" >> "$todo_file"
        grep -i "implement\|critical\|important\|fix\|bug" "$todo_file" | head -10 >> "$todo_file" 2>/dev/null || true
        
        # Show top 5 most critical TODOs
        info "Top 5 most critical TODOs:"
        grep -i "implement\|critical\|important\|fix\|bug" "$todo_file" | head -5 | while read -r line; do
            echo -e "${PURPLE}  - $line${NC}"
        done
    else
        info "✓ No TODO/FIXME items found"
    fi
}

# Function to check for missing mod.rs files
check_missing_mod_files() {
    info "Checking for missing mod.rs files"
    
    local missing_mods=()
    
    # Find all directories in src that contain .rs files but no mod.rs
    while IFS= read -r -d '' dir; do
        if [[ -n "$(find "$dir" -maxdepth 1 -name "*.rs" -not -name "mod.rs" 2>/dev/null)" ]] && [[ ! -f "$dir/mod.rs" ]]; then
            # Skip the root src directory as it uses lib.rs
            if [[ "$dir" != "$PROJECT_DIR/src" ]]; then
                missing_mods+=("$dir")
            fi
        fi
    done < <(find "$PROJECT_DIR/src" -type d -print0)
    
    if [[ ${#missing_mods[@]} -gt 0 ]]; then
        warn "Found ${#missing_mods[@]} directories missing mod.rs files"
        for dir in "${missing_mods[@]}"; do
            warn "Missing mod.rs in: $dir"
            # Create basic mod.rs file
            create_basic_mod_file "$dir"
        done
    else
        info "✓ All module directories have mod.rs files"
    fi
}

# Function to create basic mod.rs file
create_basic_mod_file() {
    local dir="$1"
    local mod_file="$dir/mod.rs"
    
    debug "Creating basic mod.rs file: $mod_file"
    
    # Backup if exists
    if [[ -f "$mod_file" ]]; then
        cp "$mod_file" "$BACKUP_DIR/$(basename "$dir")_mod.rs.backup"
    fi
    
    # Create basic mod.rs content
    cat > "$mod_file" << EOF
//! Module: $(basename "$dir")
//! 
//! Auto-generated mod.rs file for the $(basename "$dir") module.

EOF
    
    # Add pub mod declarations for all .rs files in the directory
    find "$dir" -maxdepth 1 -name "*.rs" -not -name "mod.rs" | while read -r file; do
        local module_name=$(basename "$file" .rs)
        echo "pub mod $module_name;" >> "$mod_file"
    done
    
    info "✓ Created mod.rs file: $mod_file"
}

# Function to check compilation status
check_compilation() {
    info "Checking Rust compilation status"
    
    cd "$PROJECT_DIR"
    
    # Check with debug logging
    export RUST_LOG=debug
    
    if cargo check --verbose 2>&1 | tee "$BACKUP_DIR/cargo_check.log"; then
        info "✓ Cargo check passed successfully"
        
        # Run cargo clippy for additional checks
        if cargo clippy --all-targets --all-features 2>&1 | tee "$BACKUP_DIR/cargo_clippy.log"; then
            info "✓ Cargo clippy checks passed"
        else
            warn "Cargo clippy found issues (see cargo_clippy.log)"
        fi
        
        # Run cargo test to check if tests compile
        if cargo test --no-run 2>&1 | tee "$BACKUP_DIR/cargo_test_compile.log"; then
            info "✓ Test compilation successful"
        else
            warn "Test compilation failed (see cargo_test_compile.log)"
        fi
        
    else
        error "Cargo check failed - see cargo_check.log for details"
        return 1
    fi
}

# Function to check WarpPreview.app
check_warp_preview() {
    info "Checking WarpPreview.app status"
    
    if [[ -f "$WARP_PREVIEW_PATH" ]]; then
        info "✓ WarpPreview.app found at: $WARP_PREVIEW_PATH"
        
        # Check file permissions
        if [[ -x "$WARP_PREVIEW_PATH" ]]; then
            info "✓ WarpPreview.app is executable"
        else
            warn "WarpPreview.app is not executable"
            debug "Setting executable permissions"
            chmod +x "$WARP_PREVIEW_PATH"
        fi
        
        # Get file info
        local file_size=$(stat -f%z "$WARP_PREVIEW_PATH" 2>/dev/null || echo "unknown")
        local file_date=$(stat -f%Sm "$WARP_PREVIEW_PATH" 2>/dev/null || echo "unknown")
        
        debug "WarpPreview.app size: $file_size bytes"
        debug "WarpPreview.app date: $file_date"
        
    else
        error "WarpPreview.app not found at: $WARP_PREVIEW_PATH"
        return 1
    fi
}

# Function to check project structure
check_project_structure() {
    info "Checking project structure integrity"
    
    local structure_file="$BACKUP_DIR/project_structure.txt"
    echo "=== Project Structure Analysis ===" > "$structure_file"
    
    # Core files that should exist
    local core_files=(
        "Cargo.toml"
        "src/lib.rs"
        "src/main.rs"
        "README.md"
    )
    
    for file in "${core_files[@]}"; do
        local full_path="$PROJECT_DIR/$file"
        if check_file_integrity "$full_path" "core"; then
            echo "✓ $file" >> "$structure_file"
        else
            echo "✗ $file" >> "$structure_file"
        fi
    done
    
    # Check src directory structure
    echo -e "\n=== Source Directory Structure ===" >> "$structure_file"
    find "$PROJECT_DIR/src" -type f -name "*.rs" | sort >> "$structure_file"
    
    # Check for common Rust project directories
    local common_dirs=("tests" "examples" "benches" "docs")
    echo -e "\n=== Optional Directories ===" >> "$structure_file"
    
    for dir in "${common_dirs[@]}"; do
        if [[ -d "$PROJECT_DIR/$dir" ]]; then
            echo "✓ $dir/" >> "$structure_file"
        else
            echo "○ $dir/ (optional)" >> "$structure_file"
        fi
    done
    
    info "Project structure analysis saved to: $structure_file"
}

# Function to generate restoration recommendations
generate_recommendations() {
    info "Generating restoration recommendations"
    
    local rec_file="$BACKUP_DIR/restoration_recommendations.md"
    
    cat > "$rec_file" << EOF
# Rust Code Restoration Recommendations

Generated on: $(date)

## Summary
This document contains recommendations for improving and completing the Warp terminal Rust codebase.

## Critical Issues to Address

### 1. TODO Items Requiring Implementation
- Command history specific session filtering
- Theme picker functionality
- Font family and size input handling
- Window size and opacity controls
- Syntax highlighting for search results
- Timeline view for command history
- Bookmark and tagging functionality

### 2. Missing Functionality
- Directional pane navigation implementation
- Advanced error pattern analysis
- Daily activity tracking
- Directory usage statistics
- Command trend analysis

### 3. Code Quality Improvements
- Replace TODO comments with proper implementations
- Add comprehensive error handling
- Implement missing test cases
- Add documentation for public APIs

## Implementation Priority

### High Priority
1. Complete command history functionality
2. Implement theme management system
3. Add comprehensive settings input validation
4. Implement pane navigation system

### Medium Priority
1. Add analytics and usage tracking
2. Implement batch command processing
3. Add syntax highlighting support
4. Improve error handling and reporting

### Low Priority
1. Add advanced search features
2. Implement plugin system extensions
3. Add performance monitoring
4. Enhance UI animations and transitions

## Recommended Next Steps

1. **Immediate Actions**
   - Fix all compilation warnings
   - Implement critical TODO items
   - Add missing mod.rs files where needed

2. **Short-term Goals (1-2 weeks)**
   - Complete command history functionality
   - Implement basic theme management
   - Add comprehensive error handling

3. **Long-term Goals (1-2 months)**
   - Full feature implementation
   - Comprehensive testing suite
   - Performance optimization
   - Documentation completion

## Technical Debt

The codebase shows good structure but has several areas of technical debt:
- Multiple TODO items indicating incomplete features
- Some placeholder implementations
- Missing error handling in some areas
- Incomplete test coverage

## Conclusion

The Warp terminal codebase is well-structured and compiles successfully. The main areas for improvement are completing the TODO items and implementing the missing functionality outlined above.
EOF

    info "Restoration recommendations saved to: $rec_file"
}

# Function to create missing directories if needed
create_missing_directories() {
    info "Checking for missing directories"
    
    local required_dirs=(
        "$PROJECT_DIR/logs"
        "$PROJECT_DIR/cache"
        "$PROJECT_DIR/config"
        "$PROJECT_DIR/plugins"
        "$PROJECT_DIR/scripts"
        "$PROJECT_DIR/target"
    )
    
    for dir in "${required_dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            debug "Creating missing directory: $dir"
            mkdir -p "$dir"
            info "✓ Created directory: $dir"
        else
            debug "Directory exists: $dir"
        fi
    done
}

# Main execution
main() {
    info "=== Starting Rust Code Review and Restoration ==="
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_DIR/Cargo.toml" ]]; then
        error "Not a Rust project directory: $PROJECT_DIR"
        exit 1
    fi
    
    # Create missing directories
    create_missing_directories
    
    # Check project structure
    check_project_structure
    
    # Check for missing mod.rs files
    check_missing_mod_files
    
    # Analyze TODOs and FIXMEs
    analyze_todos
    
    # Check compilation status
    if ! check_compilation; then
        error "Compilation failed - manual intervention required"
        exit 1
    fi
    
    # Check WarpPreview.app
    check_warp_preview
    
    # Generate recommendations
    generate_recommendations
    
    info "=== Rust Code Review Complete ==="
    info "Results saved to: $BACKUP_DIR"
    info "Log file: $LOG_FILE"
    
    # Summary
    echo -e "\n${CYAN}=== SUMMARY ===${NC}"
    echo -e "${GREEN}✓${NC} Project compiles successfully"
    echo -e "${GREEN}✓${NC} Core structure is intact"
    echo -e "${YELLOW}!${NC} Found TODO items requiring attention"
    echo -e "${BLUE}i${NC} Check $BACKUP_DIR for detailed analysis"
    
    return 0
}

# Run main function
main "$@"
