#!/bin/bash

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM package
echo "Building WASM package..."
wasm-pack build --target web --out-dir pkg

# Copy index.html to output directory if needed
cp index.html pkg/ 2>/dev/null || true

echo "Build complete! You can now serve the files in the pkg/ directory"
echo "For example, run: python3 -m http.server 8000 --directory pkg"
echo "Then open http://localhost:8000 in your browser"
