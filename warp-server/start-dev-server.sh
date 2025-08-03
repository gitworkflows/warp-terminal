#!/bin/bash

# Load environment variables
if [ -f .env.dev ]; then
    # Use a safer method to export variables with spaces
    while IFS='=' read -r key value; do
        # Skip comments and empty lines
        [[ $key =~ ^[[:space:]]*# ]] && continue
        [[ -z $key ]] && continue
        # Export the variable
        export "$key"="$value"
    done < .env.dev
fi

# Start the server
echo "Starting Warp development server on port $PORT..."
echo "Environment: $ENV"
echo "Log Level: $LOG_LEVEL"
echo ""

./warp-server
