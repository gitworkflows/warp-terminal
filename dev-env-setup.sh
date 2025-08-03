#!/bin/bash

# Warp Development Environment Setup
# This script sets up a local development server and backs up the database

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
WARP_SERVER_DIR="/Users/KhulnaSoft/.warp/warp-server"
DATABASE_PATH="/Users/KhulnaSoft/Library/Application Support/dev.warp.Warp-Preview/warp.sqlite"
BACKUP_DIR="/Users/KhulnaSoft/.warp/backups"
DEV_PORT="8080"
LOG_DIR="/Users/KhulnaSoft/.warp/logs"

echo -e "${BLUE}🚀 Warp Development Environment Setup${NC}"
echo "======================================"

# Function to print status messages
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Create necessary directories
echo -e "\n${BLUE}📁 Creating directories...${NC}"
mkdir -p "$BACKUP_DIR" "$LOG_DIR"
print_status "Created backup and log directories"

# 1. Database Backup
echo -e "\n${BLUE}💾 Backing up database...${NC}"
if [ -f "$DATABASE_PATH" ]; then
    BACKUP_FILENAME="warp_$(date +%Y%m%d_%H%M%S).sqlite"
    BACKUP_FULL_PATH="$BACKUP_DIR/$BACKUP_FILENAME"
    
    # Create backup using SQLite .backup command for consistency
    sqlite3 "$DATABASE_PATH" ".backup '$BACKUP_FULL_PATH'"
    
    if [ $? -eq 0 ]; then
        print_status "Database backed up to: $BACKUP_FULL_PATH"
        
        # Create a symlink to latest backup
        ln -sf "$BACKUP_FULL_PATH" "$BACKUP_DIR/latest.sqlite"
        print_status "Created symlink to latest backup"
        
        # Show backup size
        BACKUP_SIZE=$(du -h "$BACKUP_FULL_PATH" | cut -f1)
        echo "  Backup size: $BACKUP_SIZE"
    else
        print_error "Failed to backup database"
        exit 1
    fi
else
    print_warning "Database file not found at: $DATABASE_PATH"
    print_warning "This might be normal if Warp hasn't been run yet"
fi

# 2. Check if Go is installed
echo -e "\n${BLUE}🔧 Checking prerequisites...${NC}"
if ! command -v go &> /dev/null; then
    print_error "Go is not installed. Please install Go first."
    echo "Visit: https://golang.org/dl/"
    exit 1
fi

GO_VERSION=$(go version | cut -d' ' -f3)
print_status "Go is installed: $GO_VERSION"

# 3. Setup Warp Server
echo -e "\n${BLUE}🖥️  Setting up Warp server...${NC}"
cd "$WARP_SERVER_DIR"

# Install dependencies
if [ -f "go.mod" ]; then
    print_status "Installing Go dependencies..."
    go mod download
    go mod tidy
    print_status "Dependencies installed"
else
    print_warning "go.mod not found, creating new module..."
    go mod init github.com/warpdotdev/warp-server
fi

# Build the server
echo -e "\n${BLUE}🔨 Building server...${NC}"
go build -o warp-server .
if [ $? -eq 0 ]; then
    print_status "Server built successfully"
else
    print_error "Failed to build server"
    exit 1
fi

# 4. Create development configuration
echo -e "\n${BLUE}⚙️  Creating development configuration...${NC}"
cat > .env.dev << EOF
# Warp Development Environment Configuration
PORT=$DEV_PORT
ENV=development
LOG_LEVEL=debug
DATABASE_PATH=$DATABASE_PATH
BACKUP_DIR=$BACKUP_DIR
EOF

print_status "Created .env.dev configuration file"

# 5. Create systemd-style service script for development
cat > start-dev-server.sh << 'EOF'
#!/bin/bash

# Load environment variables
if [ -f .env.dev ]; then
    export $(cat .env.dev | grep -v '^#' | xargs)
fi

# Start the server
echo "Starting Warp development server on port $PORT..."
echo "Environment: $ENV"
echo "Log Level: $LOG_LEVEL"
echo ""

./warp-server
EOF

chmod +x start-dev-server.sh
print_status "Created development server startup script"

# 6. Create database inspection script
cat > inspect-db.sh << EOF
#!/bin/bash

DATABASE_PATH="$DATABASE_PATH"

if [ ! -f "\$DATABASE_PATH" ]; then
    echo "Database file not found: \$DATABASE_PATH"
    exit 1
fi

echo "Warp Database Inspector"
echo "======================"
echo "Database: \$DATABASE_PATH"
echo ""

# Show database info
echo "Database size: \$(du -h "\$DATABASE_PATH" | cut -f1)"
echo ""

# Show tables
echo "Tables in database:"
sqlite3 "\$DATABASE_PATH" ".tables"
echo ""

# Show schema for each table
echo "Database schema:"
sqlite3 "\$DATABASE_PATH" ".schema"
EOF

chmod +x inspect-db.sh
print_status "Created database inspection script"

# 7. Create backup restore script
cat > restore-backup.sh << 'EOF'
#!/bin/bash

BACKUP_DIR="/Users/KhulnaSoft/.warp/backups"
DATABASE_PATH="/Users/KhulnaSoft/Library/Application Support/dev.warp.Warp-Preview/warp.sqlite"

if [ -z "$1" ]; then
    echo "Usage: $0 <backup_file>"
    echo ""
    echo "Available backups:"
    ls -la "$BACKUP_DIR"/*.sqlite 2>/dev/null || echo "No backups found"
    exit 1
fi

BACKUP_FILE="$1"

# If it's just a filename, prepend the backup directory
if [[ "$BACKUP_FILE" != /* ]]; then
    BACKUP_FILE="$BACKUP_DIR/$BACKUP_FILE"
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Restoring database from: $BACKUP_FILE"
echo "To: $DATABASE_PATH"
echo ""

# Stop Warp if running
echo "Please close Warp application before continuing..."
read -p "Press Enter when ready..."

# Backup current database
if [ -f "$DATABASE_PATH" ]; then
    CURRENT_BACKUP="$DATABASE_PATH.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$DATABASE_PATH" "$CURRENT_BACKUP"
    echo "Current database backed up to: $CURRENT_BACKUP"
fi

# Restore from backup
cp "$BACKUP_FILE" "$DATABASE_PATH"
echo "Database restored successfully!"
EOF

chmod +x restore-backup.sh
print_status "Created backup restore script"

# 8. Check if port is available
echo -e "\n${BLUE}🌐 Checking port availability...${NC}"
if lsof -i:$DEV_PORT > /dev/null 2>&1; then
    print_warning "Port $DEV_PORT is already in use"
    echo "  Processes using port $DEV_PORT:"
    lsof -i:$DEV_PORT
    echo ""
    print_warning "You may need to stop the existing service or choose a different port"
else
    print_status "Port $DEV_PORT is available"
fi

# 9. Summary
echo -e "\n${GREEN}🎉 Development Environment Setup Complete!${NC}"
echo "=========================================="
echo ""
echo -e "${BLUE}Available Scripts:${NC}"
echo "  ./start-dev-server.sh  - Start the development server"
echo "  ./inspect-db.sh        - Inspect the Warp database"
echo "  ./restore-backup.sh    - Restore from a backup"
echo ""
echo -e "${BLUE}Server Configuration:${NC}"
echo "  Port: $DEV_PORT"
echo "  Environment: development"
echo "  Database: $DATABASE_PATH"
echo "  Backups: $BACKUP_DIR"
echo ""
echo -e "${BLUE}API Endpoints:${NC}"
echo "  http://localhost:$DEV_PORT/         - Home"
echo "  http://localhost:$DEV_PORT/health   - Health check"
echo "  http://localhost:$DEV_PORT/api/status - Server status"
echo ""

if [ -f "$BACKUP_FULL_PATH" ]; then
    echo -e "${BLUE}Latest Backup:${NC}"
    echo "  File: $BACKUP_FILENAME"
    echo "  Size: $BACKUP_SIZE"
    echo "  Path: $BACKUP_FULL_PATH"
    echo ""
fi

echo -e "${YELLOW}Next Steps:${NC}"
echo "1. cd $WARP_SERVER_DIR"
echo "2. ./start-dev-server.sh"
echo "3. Open http://localhost:$DEV_PORT in your browser"
echo ""
echo -e "${YELLOW}To start the server now, run:${NC}"
echo "cd $WARP_SERVER_DIR && ./start-dev-server.sh"
