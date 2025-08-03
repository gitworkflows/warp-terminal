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
