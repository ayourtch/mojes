# Mojes - Rust to JavaScript Transpiler Analysis

## Repository Overview

Mojes is a comprehensive Rust to JavaScript transpiler that enables writing Rust code that runs seamlessly in browser environments with full DOM API support.

### Workspace Structure

This is a Cargo workspace containing 6 packages:

1. **`mojes/`** - Main library that re-exports all components
2. **`mojes-derive/`** - Procedural macros for code generation
3. **`mojes-mojo/`** - Core transpilation engine (5,742 lines)
4. **`mojes-dom-api/`** - Complete DOM and WebAPI bindings
5. **`mojes-file/`** - File system utilities
6. **`mojes-dom-api/`** - Browser API implementations

### Technical Architecture

#### Core Transpilation (`mojes-mojo/src/lib.rs`)
- **5,742 lines** of sophisticated transpilation logic
- Uses **SWC (SuperWebCompiler)** for JavaScript AST generation
- Implements `TranspilerState` for symbol table and scope management
- Converts Rust expressions, statements, and control flow to JavaScript
- Handles complex patterns like closures, match expressions, destructuring

#### Procedural Macros (`mojes-derive/src/lib.rs`)
- `#[to_js]` - Transpiles functions to JavaScript with distributed collection
- `#[js_type]` - Generates JavaScript classes for structs/enums with JSON serialization
- `#[js_object]` - Transpiles impl blocks to JavaScript methods
- Automatic camelCase conversion for JavaScript compatibility

#### DOM API (`mojes-dom-api/src/lib.rs`)
Comprehensive browser API implementations including:
- **DOM**: Document, Element, Console, Window, History, Location
- **WebRTC**: RTCPeerConnection, MediaStream, RTCDataChannel, ICE handling
- **Storage**: localStorage, sessionStorage with JSON helpers
- **Events**: Event system, XMLHttpRequest, WebSocket
- **Media**: MediaDevices, getUserMedia, MediaStreamTrack

### Key Features

#### Advanced Transpilation Capabilities
- **Pattern Matching**: Complex match expressions, tuple patterns, wildcard patterns
- **Closures**: Parameter handling including wildcard parameters
- **Control Flow**: If/else, loops, early returns properly transpiled
- **Promises**: Sophisticated handling of Promise return patterns (with/without explicit return)
- **Type System**: JavaScript type validation generation for Rust types

#### Code Generation System
- Uses `linkme::distributed_slice` to collect generated JavaScript across modules
- AST-based transpilation ensures correct JavaScript syntax
- Mock implementations enable Rust-side testing with Boa JavaScript engine
- Automatic JavaScript function wrapper generation with type checking

#### Browser Compatibility
- Complete WebRTC implementation for peer-to-peer communication
- Full DOM manipulation capabilities
- Storage APIs (localStorage/sessionStorage)
- Event handling system
- XMLHttpRequest and WebSocket support

### Build Configuration

- **Rust Edition**: 2024 (cutting edge)
- **Core Dependencies**:
  - `syn` 2.0 with full parsing features
  - `swc_*` family for JavaScript AST manipulation
  - `linkme` for distributed code collection
  - `serde_json` for serialization
  - `boa_engine` for JavaScript testing

### Testing Strategy

Extensive test suite covering:
- **Expression transpilation** (`tests/expressions.rs`)
- **Control flow** (`tests/control_flow.rs`)
- **Pattern matching** (`tests/test_*_patterns.rs`)
- **Promise handling** (`tests/test_promise_transpilation.rs`)
- **WebRTC functionality** (`tests/webrtc_*.rs`)
- **DOM operations** (`tests/dom_tests.rs`)

Tests use Boa JavaScript engine to validate transpiled output correctness.

### Notable Technical Decisions

1. **SWC Integration**: Uses SuperWebCompiler for robust JavaScript AST generation
2. **Distributed Collection**: JavaScript code fragments collected across modules using `linkme`
3. **Mock APIs**: Full browser API implementations enable Rust-side development/testing
4. **Type-Aware Transpilation**: Rust type information drives JavaScript type validation
5. **Promise Patterns**: Sophisticated handling of implicit vs explicit returns in async code

### Development Context

- **Repository**: https://github.com/ayourtch/mojes
- **License**: MIT OR Apache-2.0
- **Status**: Active development with recent commits on closures and WebRTC features
- **Toolchain**: Rust 1.87.0, Cargo 1.87.0

## Usage Patterns

### Basic Function Transpilation
```rust
#[to_js]
fn calculate(x: i32, y: i32) -> i32 {
    x + y
}
```

### Struct/Class Generation
```rust
#[js_type]
struct Calculator {
    value: i32,
}

#[js_object]
impl Calculator {
    fn new() -> Self {
        Self { value: 0 }
    }
    
    fn add(&mut self, n: i32) {
        self.value += n;
    }
}
```

### DOM Usage
```rust
use mojes::dom::{document, console};

fn main() {
    let element = document.getElementById("my-div").unwrap();
    element.innerHTML = "Hello from Rust!".to_string();
    console.log("Page updated");
}
```

This transpiler represents a sophisticated approach to bringing Rust's safety and expressiveness to web development while maintaining full compatibility with existing JavaScript ecosystems.