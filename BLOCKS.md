# Mojes Transpiler Issues & Analysis

> **STATUS (2026-07-13): RESOLVED.** The `_rust_retval` approach recommended
> at the bottom of this document was implemented; `if`/`else` (and `match`)
> expressions used as `format!` arguments now return their branch values.
> Covered by `mojes-mojo/tests/test_gotcha_fixes.rs`
> (`format_with_if_expression_returns_value`). The "Related Issues" noted at
> the end are also fixed: constructor calls now dispatch to a custom static
> `Type.new` when one exists, and `continue` inside `.enumerate()` loops no
> longer freezes the index. This document is kept as analysis history.

## Issue: Conditional Expressions in Template Literals Return `undefined`

### Problem Description
When Rust conditional expressions (if expressions) are used in `format!` macro arguments, they transpile to JavaScript functions that return `undefined` instead of their actual values.

**Example:**
```rust
console.log(&format!("STUN candidates: {}", if has_srflx { "YES" } else { "NO" }));
```

**Current (Broken) JavaScript Output:**
```javascript
log_message(`STUN candidates: ${(function() {
    if (has_srflx) {
        "YES";                    // ❌ No return statement
    } else {
        "NO";                     // ❌ No return statement  
    }
    return undefined;             // ❌ Safety net overrides values
}).call(this)}`)
```

**Expected JavaScript Output:**
```javascript
log_message(`STUN candidates: ${(function() {
    if (has_srflx) {
        return "YES";             // ✅ Explicit return
    } else {
        return "NO";              // ✅ Explicit return
    }
}).call(this)}`)
```

### Root Cause Analysis

The issue stems from how the transpiler handles conditional expressions in different contexts:

#### 1. **Semicolon Semantics in Rust**
- **No semicolon**: `if condition { value }` → Expression that returns value
- **With semicolon**: `if condition { value; }` → Statement that returns `()`

#### 2. **Call Flow for Different Contexts**

**✅ Working: Let bindings**
```
let result = if condition { "A" } else { "B" };
  ↓
rust_block_to_js_with_state (detects no semicolon)
  ↓  
handle_if_expr_with_safety_net(..., false) // No safety net
  ↓
convert_if_to_stmt(BlockAction::Return, ...) // Branches return values
```

**❌ Broken: format! arguments**
```
format!("...", if condition { "A" } else { "B" })
  ↓
handle_format_like_macro
  ↓
parse_macro_tokens
  ↓
rust_expr_to_js_with_state
  ↓
handle_if_expr(...) // Always uses safety net
  ↓
convert_if_to_stmt(BlockAction::NoReturn, ...) + return undefined;
```

#### 3. **Key Code Locations**

**File: `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs`**

1. **Macro Dispatch (Line 2753):**
   ```rust
   "format" => handle_format_like_macro(&tokens, state),
   ```

2. **Format Macro Handler (Line 2823-2844):**
   ```rust
   fn handle_format_like_macro(token_string: &str, state: &mut TranspilerState) -> Result<js::Expr, String> {
       // ...
       let format_args: Result<Vec<_>, _> = parts
           .iter()
           .skip(1)
           .map(|arg| parse_macro_tokens(arg.trim(), state)) // ← Issue here
           .collect();
   }
   ```

3. **Parse Macro Tokens (Line 3211):**
   ```rust
   fn parse_macro_tokens(tokens: &str, state: &mut TranspilerState) -> Result<js::Expr, String> {
       if let Ok(parsed_expr) = syn::parse_str::<syn::Expr>(trimmed) {
           rust_expr_to_js_with_state(&parsed_expr, state) // ← Uses default path
       }
   }
   ```

4. **If Expression Handler (Line 934-947):**
   ```rust
   fn handle_if_expr_with_safety_net(..., add_safety_net: bool) -> Result<js::Expr, String> {
       let branch_action = if add_safety_net { block_action } else { BlockAction::Return };
       let if_stmt = convert_if_to_stmt(branch_action, if_expr, state)?;
       
       let mut stmts = vec![if_stmt];
       if add_safety_net {
           stmts.push(state.mk_return_stmt(Some(state.mk_undefined()))); // ← Safety net
       }
       Ok(state.mk_iife(stmts))
   }
   ```

5. **Block Processing with Semicolon Logic (Line 1141-1173):**
   ```rust
   Stmt::Expr(expr, semi) => {
       if semi.is_some() {
           // With semicolon - treat as statement
           let if_stmt = convert_if_to_stmt(block_action, if_expr, state)?;
           js_stmts.push(if_stmt);
       } else {
           // No semicolon - should return value
           let js_expr = handle_if_expr_with_safety_net(block_action, if_expr, state, false)?;
           // ↑ This works correctly for block contexts
       }
   }
   ```

### Attempted Solutions

#### Solution 1: Context Propagation (Partially Working)
- **Approach**: Pass value context flag through expression processing
- **Files Modified**: Added `rust_expr_to_js_with_context` and `parse_macro_tokens_with_value_context`
- **Result**: Fixed format! expressions but may have broken other functionality
- **Issue**: Too broad - affects all expression processing, not just conditional expressions

#### Solution 2: Semicolon Detection in Block Processing (Working for Let Bindings)
- **Approach**: Use Rust semicolon semantics to determine when expressions should return values
- **Files Modified**: Block processing logic in `rust_block_to_js_with_state`
- **Result**: Works for let bindings, doesn't affect format! arguments
- **Issue**: format! arguments don't go through block processing

### Key Insights

1. **Rust Semantics**: Semicolons are the authoritative indicator of whether expressions return values
2. **Different Code Paths**: Let bindings vs format! arguments use completely different processing paths
3. **Safety Net Purpose**: The `return undefined;` is intentionally added to catch cases where branches don't return values
4. **Context Matters**: If expressions need different handling based on usage context

### Testing Infrastructure

**Test File**: `/Users/ayourtch/rust/mojes/mojes/tests/test_console_format_conditional.rs`

```rust
#[to_js]
pub fn test_console_format_conditional() -> bool {
    let has_srflx = connection.has_srflx_candidates();
    log_message(&format!("STUN candidates: {}", if has_srflx { "YES" } else { "NO" }));
    // Multiple test cases with various conditional patterns
}
```

**Simple Test**: `/Users/ayourtch/rust/mojes/mojes/tests/debug_if_value_context.rs`

```rust
#[to_js]
pub fn simple_if_test() {
    let result = if true { "YES" } else { "NO" };  // ✅ Works
}

#[to_js]  
pub fn format_if_test() {
    let message = format!("Status: {}", if condition { "GOOD" } else { "BAD" }); // ❌ Broken
}
```

### Alternative Approaches to Consider

#### Approach 1: Template Literal Specific Fix
Only apply value context to expressions that appear directly in template literals, not all format! arguments.

#### Approach 2: AST Pattern Matching
Detect the specific pattern of `if expressions as format! arguments` and handle them specially.

#### Approach 3: Conditional Expression Detection
Create a function to detect when if expressions are used in value contexts vs statement contexts.

#### Approach 4: Post-Processing Fix
Fix the JavaScript after generation by detecting and correcting the specific pattern.

#### Approach 5: The `_rust_retval` Pattern (RECOMMENDED)
Use a return value accumulator variable to capture expression values from branches.

### Related Issues Identified

While investigating this issue, several other transpilation problems were discovered:

1. **Array.any() → Array.some()**: JavaScript arrays don't have `.any()` method
2. **Constructor calls**: Using `new ClassName()` instead of `ClassName.new()`
3. **Universal iteration**: Tuple destructuring needs universal collection support

### Files That Were Modified (Need Reset)

1. `/Users/ayourtch/rust/mojes/mojes-mojo/src/lib.rs`
   - Lines ~1141-1173: Block processing semicolon logic
   - Lines ~934-947: If expression handler parameterization
   - Lines ~1309-1333: Added `rust_expr_to_js_with_context`
   - Lines ~1876-1894: Added `rust_expr_to_js_with_action_and_state_value_context`
   - Lines ~2844: Modified format macro argument processing
   - Lines ~3223-3234: Added `parse_macro_tokens_with_value_context`

## The `_rust_retval` Approach (Recommended Solution)

### Concept
Instead of trying to detect value contexts or propagate flags through the entire expression pipeline, use a return value accumulator pattern that mirrors Rust's implicit return semantics.

### How It Works

**Current Broken Pattern:**
```javascript
(function() {
    if (condition) {
        "YES";                    // Expression result is lost
    } else {
        "NO";                     // Expression result is lost
    }
    return undefined;             // Always returns undefined
}).call(this)
```

**Proposed Fixed Pattern:**
```javascript
(function() {
    let _rust_retval = undefined;  // Initialize return value holder
    if (condition) {
        _rust_retval = "YES";       // Capture branch expression
    } else {
        _rust_retval = "NO";        // Capture branch expression
    }
    return _rust_retval;            // Return the captured value
}).call(this)
```

### Implementation Strategy

#### Step 1: Modify IIFE Generation
In `handle_if_expr_with_safety_net` (line ~934-947):
- Add `let _rust_retval = undefined;` as the first statement
- Change final return to `return _rust_retval;` instead of `return undefined;`

#### Step 2: Detect Last Expression in Branches
In `convert_if_to_stmt` and `rust_block_to_js_with_state`:
- For each branch block, check if the last statement is an expression without semicolon
- If yes: Convert to `_rust_retval = expression;`
- If no: Leave as normal statement

#### Step 3: Use Existing Semicolon Detection
The transpiler already has logic to detect semicolons (line ~1154-1166):
```rust
if semi.is_some() {
    // Has semicolon - treat as statement
} else {
    // No semicolon - this is a value-returning expression
    // Generate: _rust_retval = expr;
}
```

### Advantages of This Approach

1. **Minimal Code Changes**: Only affects IIFE generation and branch processing
2. **Follows Rust Semantics**: Uses semicolon presence to determine value returns
3. **No Context Propagation**: Doesn't require threading context through multiple functions
4. **Safe Fallback**: If no assignment happens, returns `undefined` (current behavior)
5. **Extensible**: Works for if, match, block expressions, etc.
6. **Localized Fix**: Changes are contained to specific functions
7. **Preserves Existing Logic**: Doesn't alter control flow or expression evaluation

### Implementation Details

#### Key Functions to Modify:

1. **`handle_if_expr_with_safety_net`** (line ~934):
   ```rust
   fn handle_if_expr_with_safety_net(...) -> Result<js::Expr, String> {
       let if_stmt = convert_if_to_stmt(branch_action, if_expr, state)?;
       
       let mut stmts = vec![];
       // Add: let _rust_retval = undefined;
       stmts.push(state.mk_var_decl("_rust_retval", Some(state.mk_undefined()), true));
       stmts.push(if_stmt);
       // Change to: return _rust_retval;
       stmts.push(state.mk_return_stmt(Some(js::Expr::Ident(state.mk_ident("_rust_retval")))));
       
       Ok(state.mk_iife(stmts))
   }
   ```

2. **`rust_block_to_js_with_state`** (line ~1089):
   - When processing the last statement in a block
   - If it's an expression without semicolon
   - Generate assignment to `_rust_retval` instead of just the expression

3. **`convert_if_to_stmt`** (line ~607):
   - Pass a flag or check if we're in a context that needs `_rust_retval`
   - Modify branch processing accordingly

### Edge Cases Handled

1. **Empty branches**: `_rust_retval` remains `undefined`
2. **Early returns**: Explicit returns bypass `_rust_retval`
3. **Nested if expressions**: Each IIFE has its own `_rust_retval`
4. **Mixed statements/expressions**: Only last expression without semicolon is captured

### Testing Strategy

1. Test simple if expressions in format!
2. Test nested if expressions
3. Test if expressions with complex branches
4. Test match expressions (should work with same approach)
5. Verify existing tests still pass

### Why This is Superior to Previous Attempts

| Previous Attempts | `_rust_retval` Approach |
|------------------|------------------------|
| Required context propagation through entire pipeline | Local changes only |
| Changed behavior of all expressions | Only affects IIFE branches |
| Complex to implement | Simple, surgical changes |
| Risk of breaking existing code | Minimal risk |
| Hard to debug | Clear, traceable pattern |

### Recommended Next Steps

1. **Reset Changes**: Revert all previous modifications to clean slate
2. **Implement `_rust_retval`**: Start with minimal implementation for if expressions
3. **Test Incrementally**: Verify with simple test cases first
4. **Extend if Successful**: Apply same pattern to match expressions, etc.
5. **Document Pattern**: Add comments explaining the `_rust_retval` pattern

The key insight is that this approach works WITH the existing transpiler architecture rather than trying to change fundamental expression processing.