<h1 align="center"><strong>Rod: The Write-Once, Validate-Anywhere Schema Library.</strong></h1>

<div align="center">
<img src="static/banner.png" alt="Rod Banner" width="100%">

[![Crates.io](https://img.shields.io/crates/v/rod-rs.svg?style=flat-square)](https://crates.io/crates/rod-rs)
[![NPM](https://img.shields.io/npm/v/rod-js.svg?style=flat-square)](https://www.npmjs.com/package/rod-js)
[![PyPI](https://img.shields.io/pypi/v/rod-py.svg?style=flat-square)](https://pypi.org/project/rod-py/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)

</div>

---

**Rod** is a high-performance schema validation engine powered by **Rust**. It compiles to native binaries, WebAssembly (Node.js/Browser), and Python extensions, allowing you to enforce the **exact same validation logic** across your entire stack.

Stop rewriting regexes in three different languages. Define it once in Rod, run it everywhere.

## 🚀 Key Features

*   **Unified Logic:** A `uuid()` check in the browser uses the exact same byte-level logic as your Rust backend. No more "Schema Drift."
*   **Zero-Copy Architecture:** Rod validates data directly in the host language's memory (V8 Heap, Python Objects, or Rust Structs) without expensive intermediate serialization.
*   **Fluent API:** A chainable builder pattern (`rod.string().min(5)`) familiar to Zod/Pydantic users.
*   **Batch Optimization:** Specialized bitmask APIs for validating massive datasets (100k+ items) with minimal FFI overhead.

---

## 📦 Installation

### Rust
```bash
cargo add rod-rs
```

### JavaScript / TypeScript
```bash
npm install rod-js
```

### Python
```bash
pip install rod-py
```

---

## ⚡ Quick Start

### 1. Rust
Uses the `rod_obj!` macro for cleaner syntax.

```rust
use rod_rs::{rod_obj, string, number, RodValidator};
use serde_json::json;

fn main() {
    // Define schema using the macro
    let user_schema = rod_obj! {
        name: string().min(3),
        email: string().email(),
        age: number().int().min(18.0)
    };

    // Data (e.g., from an API request)
    let data = json!({ 
        "name": "Rod", 
        "email": "hi@rod.rs", 
        "age": 25 
    });

    // Zero-copy wrap & validate
    let input = rod_rs::io::json::wrap(&data);
    
    match user_schema.validate(&input) {
        Ok(_) => println!("✅ Valid!"),
        Err(e) => println!("❌ Error: {:?}", e.issues),
    }
}
```

### 2. TypeScript
Requires a one-time WASM initialization.

```typescript
import { rod } from 'rod-js';

await rod.init(); // Initialize WASM

const User = rod.object({
  name: rod.string().min(3),
  email: rod.string().email(),
  tags: rod.array(rod.string()).min(1)
}).strict();

const data = { 
  name: "Rod", 
  email: "hi@rod.rs", 
  tags: ["wasm", "rust"] 
};

// 'lazy' mode reads fields on-demand without full serialization
console.log(User.parse(data, { mode: 'lazy' }));
```

### 3. Python
Native extension via PyO3.

```python
import rod

User = rod.object({
    "name": rod.string().min(3),
    "email": rod.string().email(),
    "tags": rod.array(rod.string()).min(1)
}).strict()

data = {
    "name": "Rod", 
    "email": "hi@rod.rs", 
    "tags": ["ffi", "rust"]
}

try:
    print(User.parse(data))
except ValueError as e:
    print(f"Validation failed: {e}")
```

---

## 📊 Performance & Architecture

Rod prioritizes **correctness** and **portability**.

### The "Bridge Tax"
Rod runs inside a Rust core. When used from JS or Python, there is an overhead to cross the language boundary (Foreign Function Interface).
*   **Simple Objects:** For tiny objects (e.g., `{ a: 1 }`), native libraries like Zod (JS) or Pydantic (Python) are faster because they don't pay the FFI tax.
*   **Complex Logic:** For heavy validation (UUIDs, huge arrays, complex regex), Rod often outperforms native JS/Python logic.
*   **Native Rust:** Rod is blazingly fast (~200ns per object), comparable to raw handwritten Rust checks.

### Recommendation
*   Use **Rod** when you need strict consistency between Backend/Frontend, or when processing batch data.
*   Use **Zod/Pydantic** if you are writing a single-language app and don't need shared logic.

---

## 🛠 Development

This is a monorepo managed by `just`.

| Command | Description |
| :--- | :--- |
| `just build-all` | Build Rust core, WASM bindings, and Python extension |
| `just test-all` | Run integration tests across all three languages |
| `just bench-all` | Run comparative benchmarks against Zod and Pydantic |

---

## 📄 License

MIT © [Yash Makan](https://github.com/YashMakan)