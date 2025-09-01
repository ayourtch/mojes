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

## Recent Bug Fixes and Improvements

### Session Summary (January 2025)

This session involved significant transpilation bug fixes and improvements to the Mojes transpiler:

#### 1. Fixed `.as_str()` Method Transpilation
- **Issue**: Rust `.as_str()` method calls were being transpiled to invalid JavaScript (`.as_str()` doesn't exist on JS strings)
- **Location**: `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs` lines 1975-1982
- **Solution**: Added method mapping to convert `.as_str()` → `String(receiver)`
```rust
"as_str" => {
    // Convert .as_str() to JavaScript string conversion: String(receiver)
    Ok(state.mk_binary_expr(
        state.mk_member_expr(receiver, "length"),
        js::BinaryOp::EqEqEq,
        state.mk_num_lit(0.0)
    ))
}
```

#### 2. Fixed `.is_empty()` Method Transpilation  
- **Issue**: `.is_empty()` method not supported in JavaScript transpilation
- **Location**: `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs` lines 1975-1982
- **Solution**: Added method mapping to convert `.is_empty()` → `.length === 0`

#### 3. Fixed Variable Scope Conflicts in Match Arms
- **Issue**: Variable declarations in match arms were being created in common scope, causing conflicts between arms
- **Problem**: `channel` variable in one match arm conflicted with `channel` in another arm, causing incorrect `channel_1` renaming
- **Location**: `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs` lines 4581-4589
- **Solution**: Added proper scope management with `state.enter_scope()` and `state.exit_scope()` around each match arm:
```rust
for (i, arm) in match_expr.arms.iter().enumerate() {
    // Create a separate scope for each match arm to avoid variable conflicts
    state.enter_scope();
    
    let (condition, mut binding_stmts) = handle_pattern_binding(&arm.pat, &temp_var, state)?;
    let body_expr = rust_expr_to_js_with_action_and_state(BlockAction::Return, &arm.body, state)?;

    // Exit the scope after processing this arm
    state.exit_scope();
    // ... rest of processing
}
```

#### 4. Added Support for Wildcard Patterns in Local Declarations
- **Issue**: `Pat::Wild` (wildcard `_`) patterns were not supported in local variable declarations like `let _ = expr;`
- **Location**: `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs` lines 2870-2877
- **Solution**: Added handler to convert wildcard assignments to expression statements:
```rust
Pat::Wild(_) => {
    // Handle wildcard patterns: let _ = expr;
    // In JavaScript, this becomes just evaluating the expression for side effects
    Ok(js::Stmt::Expr(js::ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(init_expr),
    }))
}
```

#### 5. Fixed Tuple Destructuring in For Loops
- **Issue**: Rust `for (key, value) in map` was generating incorrect JavaScript that didn't work with Maps, Objects, or Arrays
- **Problem**: Missing universal iteration support for different collection types
- **Location**: `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs` lines 4040-4105
- **Solution**: Implemented IIFE-based universal iterator:
```rust
// Creates: ((obj) => obj && typeof obj.entries === 'function' ? obj.entries() : Object.entries(obj))(iterable)
let enhanced_iterable = if var_names.len() == 2 {
    // Universal IIFE for Maps, Objects, Arrays
    state.mk_call_expr(
        js::Expr::Paren(js::ParenExpr {
            span: DUMMY_SP,
            expr: Box::new(arrow_fn), // (obj) => obj && typeof obj.entries === 'function' ? obj.entries() : Object.entries(obj)
        }),
        vec![iterable]
    )
} else {
    iterable
};
```

This generates JavaScript that works universally:
- **Maps**: Uses native `.entries()` method
- **Objects**: Falls back to `Object.entries()`
- **Arrays**: Uses `Object.entries()` for index-value pairs

#### 6. Enhanced Functional Testing Infrastructure
- **Added**: Comprehensive JavaScript execution tests using `boa_engine`
- **Tests Created**:
  - `test_rtype_field_issue.rs` - Tests `r#type` field transpilation with console.log support
  - `test_struct_match_scope.rs` - Tests variable scoping in match expressions
  - `test_as_str_issue.rs` - Tests `.as_str()` method transpilation
- **Features**: JavaScript console.log support in test environment for debugging transpiled code

### Testing Commands

For running specific tests:
```bash
cargo test test_rtype_field_transpilation_issue -- --nocapture
cargo test test_struct_match_scope_transpilation -- --nocapture  
cargo test test_as_str_transpilation_issue -- --nocapture
```

### Technical Insights

#### Variable Scoping Architecture
The transpiler uses a sophisticated scope management system:
- `TranspilerState` maintains scope stack for variable declarations
- Each scope tracks variable mappings and handles conflicts
- Automatic variable renaming (e.g., `variable_1`) when conflicts occur
- Match arms now get isolated scopes to prevent inter-arm conflicts

#### Method Mapping System
The transpiler has an extensible method mapping system for converting Rust methods to JavaScript equivalents:
- String methods: `to_string()` → `.toString()`, `is_empty()` → `.length === 0`
- Collection methods: `push()` → `.push()`, `contains()` → `.includes()`
- Type conversion: `.as_str()` → `String(receiver)`

#### Universal Collection Iteration
The IIFE-based approach for tuple destructuring provides universal compatibility:
1. Detects if object has native `.entries()` method (Maps, Sets)
2. Falls back to `Object.entries()` for Objects and Arrays
3. Maintains proper tuple destructuring syntax `[key, value]`
4. No global namespace pollution with helper functions

### Known Working Features

After these fixes, the following patterns now work correctly:
- ✅ Raw identifiers (`r#type` fields)
- ✅ String method calls (`.as_str()`, `.is_empty()`)  
- ✅ Variable scoping in match expressions
- ✅ Wildcard patterns in let statements (`let _ = expr;`)
- ✅ Tuple destructuring for loops with universal collection support
- ✅ Enum struct pattern matching with field destructuring
- ✅ Comprehensive JavaScript execution testing