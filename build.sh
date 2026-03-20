#!/usr/bin/env bash
set -euo pipefail

echo "Building Glider for WASM..."

# Build WASM release
cargo build --target wasm32-unknown-unknown --release

# Create dist directory
mkdir -p dist

# Copy WASM output
cp target/wasm32-unknown-unknown/release/glider.wasm dist/

# Copy HTML
cp index.html dist/

# Copy background texture
mkdir -p dist/assets/pixel_skies/pixel_skies_1920x1080
cp "assets/pixel_skies/pixel_skies_1920x1080/demo06_PixelSky_1920x1080.png" \
   "dist/assets/pixel_skies/pixel_skies_1920x1080/"

# Copy spritesheet (needed for player and terrain tiles)
mkdir -p dist/assets/kenney_block-pack/Spritesheet
cp assets/kenney_block-pack/Spritesheet/blockPack_spritesheet.png \
   dist/assets/kenney_block-pack/Spritesheet/

# Download mq_js_bundle.js if not present
if [ ! -f dist/mq_js_bundle.js ]; then
  curl -fsSL https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js \
    -o dist/mq_js_bundle.js
fi

echo "✓ dist/ ready for deployment"
echo "WASM size: $(du -h dist/glider.wasm | cut -f1)"
