#!/bin/bash

DATABASE_PATH="/Users/KhulnaSoft/Library/Application Support/dev.warp.Warp-Preview/warp.sqlite"

if [ ! -f "$DATABASE_PATH" ]; then
    echo "Database file not found: $DATABASE_PATH"
    exit 1
fi

echo "Warp Database Inspector"
echo "======================"
echo "Database: $DATABASE_PATH"
echo ""

# Show database info
echo "Database size: $(du -h "$DATABASE_PATH" | cut -f1)"
echo ""

# Show tables
echo "Tables in database:"
sqlite3 "$DATABASE_PATH" ".tables"
echo ""

# Show schema for each table
echo "Database schema:"
sqlite3 "$DATABASE_PATH" ".schema"
