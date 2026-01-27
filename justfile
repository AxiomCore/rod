# List all commands
default:
    @just --list

# --- Internal Helpers ---

# Ensure the virtual environment exists and has maturin installed
setup-python:
    @if [ ! -d ".venv" ]; then \
        echo "🐍 Creating virtual environment..."; \
        python3 -m venv .venv; \
    fi
    @.venv/bin/pip install maturin -q

# --- Build Commands ---

# Build everything
build-all: build-wasm build-js build-python

# Build WASM bindings
build-wasm:
    bash scripts/build-wasm.sh

# Build TypeScript package
build-js:
    npm run build -w languages/js

# Build and install the Python extension into the local .venv
build-python: setup-python
    cd languages/python && ../../.venv/bin/maturin develop --release

# --- Benchmark Commands ---

# Run benchmarks comparing Zod and Rod
bench-js: build-wasm build-js
	@echo "🚀 Setting up benchmarks..."
	@cd languages/js/bench && npm install --quiet
	@echo "📊 Running benchmarks..."
	@node languages/js/bench/index.js

bench-rust:
	cd core && cargo bench

bench-python: build-python
	@echo "🚀 Setting up Python benchmarks..."
	@.venv/bin/pip install -r languages/python/bench/requirements.txt -q
	@echo "📊 Running benchmarks..."
	@.venv/bin/python languages/python/bench/main.py

bench-all: bench-rust bench-js bench-python

# --- Test Commands ---

# Run every test in the monorepo
test-all: test-rust test-js test-python

# Run core Rust logic tests
test-rust:
    cargo test --workspace

# Run TypeScript integration tests
test-js:
    npm test -w languages/js

# Run Python integration tests using the .venv python
test-python: build-python
    .venv/bin/python languages/python/tests/test_rod.py


# --- Publishing Commands ---

# Publish everything to all registries
publish-all: publish-rust publish-js publish-python

# Publish the core Rust library to crates.io (as rod-rs)
publish-rust:
    @echo "📦 Publishing rod-rs to crates.io..."
    cd core && cargo publish

# Publish the JS/TS package to NPM (as rod-js)
publish-js: build-wasm build-js
    @echo "📦 Publishing rod-js to NPM..."
    cd languages/js && npm publish --access public

# Publish the Python extension to PyPI (as rod-py)
publish-python: setup-python
    @echo "📦 Publishing rod-py to PyPI via Maturin..."
    cd languages/python && ../../.venv/bin/maturin publish

# --- Documentation ---

docs-dev:
    npm run dev -w docs

# --- Cleanup ---

clean:
    cargo clean
    rm -rf target/
    rm -rf .venv/
    rm -rf languages/js/dist/
    rm -rf languages/js/wasm/