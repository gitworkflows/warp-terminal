#!/bin/bash

# Warp Terminal Code Analysis & Implementation Script
# This script provides easy access to the comprehensive code analysis tool

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
PROJECT_PATH="/Users/KhulnaSoft/.warp"
PYTHON_SCRIPT="$PROJECT_PATH/code_analysis_tool.py"

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}$1${NC}"
}

# Check if Python is available
check_python() {
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is required but not installed."
        exit 1
    fi
}

# Check if the project directory exists
check_project() {
    if [ ! -d "$PROJECT_PATH" ]; then
        print_error "Project directory not found: $PROJECT_PATH"
        exit 1
    fi
    
    if [ ! -f "$PROJECT_PATH/Cargo.toml" ]; then
        print_error "Cargo.toml not found in project directory. This doesn't appear to be a Rust project."
        exit 1
    fi
}

# Check if Rust tools are available
check_rust_tools() {
    local missing_tools=()
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if ! command -v rustc &> /dev/null; then
        missing_tools+=("rustc")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_warning "Some Rust tools are missing: ${missing_tools[*]}"
        print_warning "Some analysis features may not work properly."
        print_warning "Install Rust from: https://rustup.rs/"
    fi
}

# Install additional cargo tools if needed
install_cargo_tools() {
    print_status "Checking for additional cargo tools..."
    
    # List of useful tools for analysis
    local tools=("cargo-audit" "cargo-outdated" "cargo-tree")
    
    for tool in "${tools[@]}"; do
        if ! cargo "$tool" --version &> /dev/null; then
            print_warning "$tool not found. Some analysis features may be limited."
            read -p "Would you like to install $tool? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                print_status "Installing $tool..."
                cargo install "$tool" || print_warning "Failed to install $tool"
            fi
        else
            print_success "$tool is available"
        fi
    done
}

# Show usage information
show_usage() {
    cat << EOF
${PURPLE}Warp Terminal Code Analysis Tool${NC}

${CYAN}USAGE:${NC}
    $0 [OPTIONS] [COMMAND]

${CYAN}COMMANDS:${NC}
    analyze              Run full code analysis
    quick               Run quick analysis (structure, dependencies, code_quality)
    security            Run security-focused analysis  
    performance         Run performance analysis
    test                Run test analysis
    architecture        Run architecture analysis
    
    build               Build the project
    test-run            Run tests
    clean               Clean build artifacts
    fmt                 Format code
    clippy              Run clippy lints
    update              Update dependencies
    doc                 Generate documentation
    
    interactive         Start interactive mode
    report              Generate analysis report (requires previous analysis)

${CYAN}OPTIONS:${NC}
    -h, --help          Show this help message
    -p, --project PATH  Set project path (default: $PROJECT_PATH)
    -f, --format FORMAT Output format: json|markdown (default: json)
    -o, --output FILE   Output file for report
    -v, --verbose       Verbose output
    --install-tools     Install missing cargo tools
    --check-deps        Check and install dependencies

${CYAN}EXAMPLES:${NC}
    $0 analyze                          # Run full analysis
    $0 quick -f markdown               # Quick analysis with markdown output
    $0 security -o security_report.json # Security analysis to file
    $0 build test fmt                  # Run multiple commands
    $0 interactive                     # Start interactive mode

${CYAN}ANALYSIS TYPES:${NC}
    - structure:     Project structure and organization
    - dependencies:  Dependency analysis and security
    - code_quality:  Code quality metrics and linting
    - security:      Security vulnerabilities and unsafe code
    - performance:   Performance characteristics and optimization
    - tests:         Test coverage and organization
    - architecture:  Software architecture and design patterns

EOF
}

# Run analysis with specified types
run_analysis() {
    local analysis_types=("$@")
    local cmd_args=()
    
    if [ ${#analysis_types[@]} -eq 0 ]; then
        analysis_types=("all")
    fi
    
    cmd_args+=(--analysis "${analysis_types[@]}")
    
    if [ ! -z "$OUTPUT_FORMAT" ]; then
        cmd_args+=(--output-format "$OUTPUT_FORMAT")
    fi
    
    if [ ! -z "$OUTPUT_FILE" ]; then
        cmd_args+=(--output-file "$OUTPUT_FILE")
    fi
    
    if [ ! -z "$PROJECT_PATH_OVERRIDE" ]; then
        cmd_args+=(--project-path "$PROJECT_PATH_OVERRIDE")
    fi
    
    print_header "🚀 Running Warp Terminal Code Analysis"
    python3 "$PYTHON_SCRIPT" "${cmd_args[@]}"
}

# Run project commands
run_commands() {
    local commands=("$@")
    local cmd_args=(--commands "${commands[@]}")
    
    if [ ! -z "$PROJECT_PATH_OVERRIDE" ]; then
        cmd_args+=(--project-path "$PROJECT_PATH_OVERRIDE")
    fi
    
    print_header "🔧 Running Project Commands: ${commands[*]}"
    python3 "$PYTHON_SCRIPT" "${cmd_args[@]}"
}

# Run interactive mode
run_interactive() {
    local cmd_args=(--interactive)
    
    if [ ! -z "$PROJECT_PATH_OVERRIDE" ]; then
        cmd_args+=(--project-path "$PROJECT_PATH_OVERRIDE")
    fi
    
    print_header "🎮 Starting Interactive Mode"
    python3 "$PYTHON_SCRIPT" "${cmd_args[@]}"
}

# Main script logic
main() {
    local commands=()
    local analysis_types=()
    local project_commands=()
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -p|--project)
                PROJECT_PATH_OVERRIDE="$2"
                PYTHON_SCRIPT="$PROJECT_PATH_OVERRIDE/code_analysis_tool.py"
                shift 2
                ;;
            -f|--format)
                OUTPUT_FORMAT="$2"
                shift 2
                ;;
            -o|--output)
                OUTPUT_FILE="$2"
                shift 2
                ;;
            -v|--verbose)
                set -x
                shift
                ;;
            --install-tools)
                install_cargo_tools
                exit 0
                ;;
            --check-deps)
                check_python
                check_project
                check_rust_tools
                install_cargo_tools
                exit 0
                ;;
            analyze)
                analysis_types+=("all")
                shift
                ;;
            quick)
                analysis_types+=("structure" "dependencies" "code_quality")
                shift
                ;;
            security)
                analysis_types+=("security")
                shift
                ;;
            performance)
                analysis_types+=("performance")
                shift
                ;;
            test)
                analysis_types+=("tests")
                shift
                ;;
            architecture)
                analysis_types+=("architecture")
                shift
                ;;
            build|test-run|clean|fmt|clippy|update|doc)
                case $1 in
                    test-run) project_commands+=("test") ;;
                    *) project_commands+=("$1") ;;
                esac
                shift
                ;;
            interactive)
                commands+=("interactive")
                shift
                ;;
            report)
                commands+=("report")
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Pre-flight checks
    check_python
    check_project
    check_rust_tools
    
    # Make sure the Python script exists
    if [ ! -f "$PYTHON_SCRIPT" ]; then
        print_error "Python analysis script not found: $PYTHON_SCRIPT"
        print_error "Make sure you're running this from the correct directory."
        exit 1
    fi
    
    # Make the Python script executable
    chmod +x "$PYTHON_SCRIPT"
    
    # Execute commands
    if [[ " ${commands[@]} " =~ " interactive " ]]; then
        run_interactive
    elif [[ " ${commands[@]} " =~ " report " ]]; then
        # Generate report from existing analysis
        local cmd_args=()
        if [ ! -z "$OUTPUT_FORMAT" ]; then
            cmd_args+=(--output-format "$OUTPUT_FORMAT")
        fi
        if [ ! -z "$OUTPUT_FILE" ]; then
            cmd_args+=(--output-file "$OUTPUT_FILE")
        fi
        print_header "📄 Generating Analysis Report"
        python3 "$PYTHON_SCRIPT" "${cmd_args[@]}"
    elif [ ${#project_commands[@]} -gt 0 ]; then
        run_commands "${project_commands[@]}"
    elif [ ${#analysis_types[@]} -gt 0 ]; then
        run_analysis "${analysis_types[@]}"
    else
        # Default: run full analysis
        print_status "No specific command provided. Running full analysis..."
        run_analysis "all"
    fi
    
    print_success "Operation completed successfully!"
}

# Handle script interruption
trap 'print_error "Script interrupted"; exit 1' INT TERM

# Run main function with all arguments
main "$@"
