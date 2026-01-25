#!/bin/bash
set -e

# 1. Ensure we are running from the project root
# This gets the directory of the script and moves one level up
cd "$(dirname "$0")/.."

echo "🏗️  Starting WASM Build Pipeline..."

# 2. Define paths
WASM_CRATE="bindings/rod-wasm"
JS_WASM_DIR="languages/js/wasm"

# 3. Clean previous artifacts in the JS folder
rm -rf "$JS_WASM_DIR"
mkdir -p "$JS_WASM_DIR/web"
mkdir -p "$JS_WASM_DIR/node"

# 4. Build for Browser (ES Modules)
# Note: --out-dir is relative to the crate path unless absolute
echo "🦀 Building WASM for Web (ESM)..."
wasm-pack build "$WASM_CRATE" --target web --out-dir "../../$JS_WASM_DIR/web" --release

# 5. Build for Node.js (CommonJS)
echo "🦀 Building WASM for Node.js (CJS)..."
wasm-pack build "$WASM_CRATE" --target nodejs --out-dir "../../$JS_WASM_DIR/node" --release

# 6. Cleanup generated .gitignore files 
# wasm-pack generates these by default, but they interfere with NPM publishing
rm -f "$JS_WASM_DIR/web/.gitignore"
rm -f "$JS_WASM_DIR/node/.gitignore"

# 7. Cleanup old relic folder if it exists
if [ -d "rod-js" ]; then
    echo "🧹 Cleaning up legacy rod-js folder..."
    rm -rf rod-js
fi

echo "✅ WASM Build Complete. Artifacts are in $JS_WASM_DIR"