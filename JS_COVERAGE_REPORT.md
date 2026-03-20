# Mojes JavaScript Coverage Report

**Date:** 2026-03-20
**Transpiler version:** Based on `mojes-mojo/src/lib.rs` (~6,660 lines)
**Scope:** Assessment of JavaScript/ECMAScript features and Web APIs covered by the Mojes Rust-to-JS transpiler

---

## Status Legend

| Symbol | Meaning |
|--------|---------|
| Supported | Fully working in the transpiler |
| Partial | Some aspects work; limitations noted |
| Not supported | Not implemented |
| N/A | Not applicable to the Rust-to-JS transpilation model |

---

## 1. Core JavaScript Language Features

### 1.1 Variables and Declarations

| Feature | Status | Notes |
|---------|--------|-------|
| `let` | Supported | Used for mutable Rust variables (`let mut`) |
| `const` | Supported | Used for immutable Rust variables (`let`) |
| `var` | Not supported | Never generated; `let`/`const` used exclusively (correct modern JS) |
| Hoisting semantics | N/A | Rust has no hoisting; `let`/`const` avoids the issue |
| Temporal dead zone | N/A | Handled implicitly by `let`/`const` usage |

### 1.2 Data Types and Literals

| Feature | Status | Notes |
|---------|--------|-------|
| Numbers (integer) | Supported | Rust integers parsed as `f64` |
| Numbers (float) | Supported | Direct transpilation |
| Strings (double-quoted) | Supported | With escape handling (`\n`, `\t`, `\r`, `\\`, `\"`) |
| Strings (single-quoted) | Supported | Available via `mk_str_lit_single_quote` |
| Booleans | Supported | `true`/`false` map directly |
| `null` | Supported | Rust `None` maps to `null` |
| `undefined` | Supported | Used in various places (uninitialized vars, return values) |
| Template literals (backticks) | Supported | Generated for `format!` macro and string concatenation |
| BigInt | Not supported | No Rust equivalent mapping |
| Symbol | Not supported | No Rust equivalent mapping |
| RegExp literals | Not supported | No transpilation from Rust regex to JS regex |
| Char literals | Supported | Transpiled to single-character strings |

### 1.3 Operators

#### Arithmetic Operators

| Operator | Status | Notes |
|----------|--------|-------|
| `+` (addition) | Supported | Also handles string concatenation via template literals |
| `-` (subtraction) | Supported | |
| `*` (multiplication) | Supported | |
| `/` (division) | Supported | |
| `%` (modulo) | Supported | |
| `**` (exponentiation) | Not supported | No direct Rust equivalent (`pow()` method not mapped) |
| `++` (increment) | Partial | Generated in `for` loop enumerate patterns only |
| `--` (decrement) | Not supported | No Rust equivalent |

#### Comparison Operators

| Operator | Status | Notes |
|----------|--------|-------|
| `===` (strict equal) | Supported | Rust `==` maps to `===` (strict equality) |
| `!==` (strict not equal) | Supported | Rust `!=` maps to `!==` |
| `<` | Supported | |
| `>` | Supported | |
| `<=` | Supported | |
| `>=` | Supported | |
| `==` (loose equal) | Not supported | Never generated (good practice) |
| `!=` (loose not equal) | Not supported | Never generated (good practice) |

#### Logical Operators

| Operator | Status | Notes |
|----------|--------|-------|
| `&&` | Supported | |
| `\|\|` | Supported | |
| `!` | Supported | |
| `??` (nullish coalescing) | Not supported | Could map from Rust `unwrap_or` patterns |
| `?.` (optional chaining) | Not supported | Could map from Rust `?` or `.map()` chains |

#### Bitwise Operators

| Operator | Status | Notes |
|----------|--------|-------|
| `&` | Supported | |
| `\|` | Supported | |
| `^` | Supported | |
| `~` (NOT) | Not supported | No Rust equivalent in the `!` mapping |
| `<<` | Supported | |
| `>>` | Supported | |
| `>>>` (unsigned right shift) | Not supported | No Rust equivalent |

#### Assignment Operators

| Operator | Status | Notes |
|----------|--------|-------|
| `=` | Supported | |
| `+=` | Supported | |
| `-=` | Supported | |
| `*=` | Supported | |
| `/=` | Supported | |
| `%=` | Supported | |
| `&=` | Supported | |
| `\|=` | Supported | |
| `^=` | Supported | |
| `<<=` | Supported | |
| `>>=` | Supported | |
| `>>>=` | Not supported | No Rust equivalent |
| `??=` | Not supported | |
| `&&=` | Not supported | |
| `\|\|=` | Not supported | |

#### Other Operators

| Operator | Status | Notes |
|----------|--------|-------|
| `typeof` | Partial | Generated in IIFE type checks (e.g., for `len()`, `get()`) |
| `instanceof` | Not supported | No Rust equivalent currently mapped |
| `in` | Not supported | No Rust equivalent currently mapped |
| `delete` | Supported | Used in `remove()` method transpilation for objects |
| `void` | Not supported | |
| Comma operator | Not supported | |
| Spread (`...`) | Not supported | Could map from Rust `..` rest patterns |
| Ternary (`? :`) | Supported | Generated for conditional method dispatch (IIFE patterns) |

### 1.4 Control Flow

| Feature | Status | Notes |
|---------|--------|-------|
| `if` / `else if` / `else` | Supported | Full support including nested chains |
| `if` as expression (value-returning) | Supported | Uses IIFE with `_rust_retval` pattern |
| `switch` / `case` | Partial | Generated for enum `toJSON` methods; `match` uses if/else chains instead |
| `for...of` | Supported | Primary loop output for Rust `for x in iter` |
| `for...in` | Not supported | Not generated; `for...of` used instead |
| Traditional `for (;;)` | Partial | Generated only for enumerate patterns |
| `while` | Supported | Including `while true` and `while let` patterns |
| `do...while` | Not supported | No Rust equivalent |
| `break` | Partial | Generates `break` in while-let loops; break-with-value is limited |
| `continue` | Partial | Recognized but transpiled to `undefined` (incomplete) |
| `return` | Supported | Full support in functions and methods |
| Labels (`label:`) | Not supported | Not transpiled from Rust labeled blocks/loops |
| `throw` | Partial | Generated in type validation code only |

### 1.5 Functions

| Feature | Status | Notes |
|---------|--------|-------|
| Function declarations | Supported | Both top-level and nested |
| Function expressions | Supported | Used in method assignments |
| Arrow functions | Supported | Generated for closures, IIFEs, callbacks |
| Default parameters | Not supported | Rust default values not transpiled |
| Rest parameters (`...args`) | Not supported | No Rust equivalent mapping |
| Spread in calls (`fn(...arr)`) | Not supported | |
| Generator functions (`function*`) | Not supported | No Rust equivalent |
| `async` functions | Supported | Via `Expr::Async` handling |
| `await` | Supported | Via `Expr::Await` handling |
| IIFE (Immediately Invoked Function Expression) | Supported | Core pattern used throughout for blocks-as-expressions |
| Closures / Arrow functions | Supported | Rust closures become JS arrow functions |
| Wildcard parameters (`_`) | Supported | Generates `_unused_N` placeholder names |
| Typed closure parameters | Supported | Type annotations stripped |

### 1.6 Classes

| Feature | Status | Notes |
|---------|--------|-------|
| `class` declarations | Supported | Generated from `#[js_type]` structs |
| `constructor` | Supported | Auto-generated from struct fields |
| Instance methods | Supported | Via `StructName.prototype.methodName` assignment |
| Static methods | Supported | Via `StructName.methodName` assignment |
| Getters (`get prop()`) | Not supported | |
| Setters (`set prop()`) | Not supported | |
| Private fields (`#field`) | Not supported | |
| `extends` / inheritance | Not supported | No Rust trait-to-class inheritance mapping |
| `super` | Not supported | |
| Computed property names | Not supported | |
| Class fields | Not supported | Initialized in constructor instead |

### 1.7 Objects

| Feature | Status | Notes |
|---------|--------|-------|
| Object literals | Supported | Generated from Rust struct expressions |
| Property shorthand (`{x}` for `{x: x}`) | Not supported | Always uses key-value syntax |
| Computed property names | Not supported | |
| Method shorthand | Not supported | |
| Spread in objects (`{...obj}`) | Not supported | |
| Object destructuring (`const {a, b} = obj`) | Supported | From Rust struct destructuring patterns |
| Nested destructuring | Not supported | |
| Destructuring with defaults | Not supported | |
| Getters/setters in literals | Not supported | |

### 1.8 Arrays

| Feature | Status | Notes |
|---------|--------|-------|
| Array literals | Supported | From Rust array/vec expressions |
| Array destructuring (`const [a, b] = arr`) | Supported | From Rust tuple destructuring |
| `Array.from()` | Supported | Used for ranges and repeat expressions |
| Spread in arrays (`[...arr]`) | Not supported | |
| Nested destructuring | Not supported | |

### 1.9 Error Handling

| Feature | Status | Notes |
|---------|--------|-------|
| `try` / `catch` / `finally` | Not supported | Rust `?` operator uses IIFE with `.error` check instead |
| `throw` | Partial | Only in generated type validation code |
| `Error` constructor | Not supported | |
| Custom error types | Not supported | |
| Error chaining | Not supported | |

### 1.10 Modules

| Feature | Status | Notes |
|---------|--------|-------|
| `import` | Not supported | Uses `linkme::distributed_slice` for code collection instead |
| `export` | Not supported | |
| Dynamic `import()` | Not supported | |
| `import.meta` | Not supported | |
| Top-level `await` | Not supported | |

### 1.11 Iterators and Generators

| Feature | Status | Notes |
|---------|--------|-------|
| `for...of` | Supported | Generated from Rust `for` loops |
| `Symbol.iterator` | Not supported | |
| Generator functions | Not supported | |
| `yield` / `yield*` | Not supported | |
| Async generators | Not supported | |
| `for await...of` | Not supported | |

### 1.12 Destructuring

| Feature | Status | Notes |
|---------|--------|-------|
| Array destructuring in `let` | Supported | From Rust tuple patterns |
| Object destructuring in `let` | Supported | From Rust struct patterns |
| Array destructuring in `for...of` | Supported | With universal IIFE for Maps/Objects |
| Default values in destructuring | Not supported | |
| Rest element in destructuring | Not supported | |
| Nested destructuring | Not supported | |
| Parameter destructuring | Not supported | |

### 1.13 Template Literals

| Feature | Status | Notes |
|---------|--------|-------|
| Basic template literals | Supported | Generated from `format!` macro |
| Expression interpolation `${expr}` | Supported | Format placeholders `{}` become interpolations |
| Tagged templates | Not supported | |
| Nested template literals | Not supported | |

### 1.14 Other Language Features

| Feature | Status | Notes |
|---------|--------|-------|
| `Proxy` | Not supported | |
| `Reflect` | Not supported | |
| `WeakMap` | Not supported | |
| `WeakSet` | Not supported | |
| `WeakRef` | Not supported | |
| `FinalizationRegistry` | Not supported | |
| `Map` | Partial | `HashMap::new()` generates `{}` (plain object), not `new Map()` |
| `Set` | Not supported | `HashSet` recognized in type formatting but no transpilation |
| Regular expressions | Not supported | |
| Labeled statements | Not supported | |
| `with` statement | Not supported | (deprecated in JS anyway) |
| Comma expressions | Not supported | |
| Computed member access `obj[expr]` | Supported | From Rust index expressions |

---

## 2. Async JavaScript

| Feature | Status | Notes |
|---------|--------|-------|
| `async` function | Supported | Via `Expr::Async` |
| `await` expression | Supported | Via `Expr::Await` |
| `Promise` constructor | Partial | Rust mock `Promise` struct available; `.resolve()`, `.reject()` |
| `Promise.then()` | Partial | Mock available in Rust; transpiles to `.then()` |
| `Promise.catch()` | Partial | Mock available in Rust; transpiles to `.catch()` |
| `Promise.finally()` | Not supported | |
| `Promise.all()` | Not supported | |
| `Promise.allSettled()` | Not supported | |
| `Promise.race()` | Not supported | |
| `Promise.any()` | Not supported | |
| Top-level `await` | Not supported | |
| Async iterators | Not supported | |
| `AsyncGenerator` | Not supported | |

---

## 3. Rust-to-JS Type Mappings

| Rust Type | JavaScript Output | Notes |
|-----------|------------------|-------|
| `i8`..`i64`, `u8`..`u64`, `f32`, `f64` | `number` | All numeric types collapse to JS number |
| `bool` | `boolean` | Direct mapping |
| `String`, `&str` | `string` | Direct mapping |
| `Vec<T>` | `Array` / `[]` | `Vec::new()` becomes `[]` |
| `HashMap<K,V>` / `BTreeMap` | `Object` / `{}` | `HashMap::new()` becomes `{}` (not `new Map()`) |
| `Option<T>` | `T \| null \| undefined` | `Some(v)` becomes `v`, `None` becomes `null` |
| `Result<T,E>` | `{ok: T}` or `{error: E}` | Custom envelope pattern |
| Structs | `class` | Via `#[js_type]` proc macro |
| Enums (unit variants) | Object with string values | Via `#[js_type]` proc macro |
| Enums (data variants) | Object with factory functions | Factory returns `{type, ...fields}` |
| Tuples | Arrays | `(a, b)` becomes `[a, b]` |
| `self` | `this` | Direct mapping |
| `Self` | Current struct name | Resolved from `TranspilerState` |
| Closures | Arrow functions | |
| References (`&`, `&mut`) | No-op (with comment) | References are transparent in JS |
| Dereference (`*`) | No-op | |
| Cast (`as`) | `Number()`, `String()`, `Boolean()` | For primitive type casts |

---

## 4. Rust Method-to-JS Method Mappings

| Rust Method | JavaScript Output | Notes |
|-------------|------------------|-------|
| `.len()` / `.count()` | IIFE: `obj.length` or `Object.keys(obj).length` | Universal for arrays and objects |
| `.clone()` | No-op (returns receiver) | |
| `.as_str()` | `String(receiver)` | |
| `.push()` | `.push()` | |
| `.pop()` | `.pop()` | |
| `.contains()` | `.includes()` | |
| `.contains_key()` | IIFE: `.has()` or `.hasOwnProperty()` | Universal for Map/Object |
| `.to_string()` | `.toString()` | |
| `.to_uppercase()` | `.toUpperCase()` | |
| `.to_lowercase()` | `.toLowerCase()` | |
| `.trim()` | `.trim()` | |
| `.trim_start()` | `.trimStart()` | |
| `.trim_end()` | `.trimEnd()` | |
| `.is_empty()` | `.length === 0` | |
| `.starts_with()` | `.startsWith()` | |
| `.ends_with()` | `.endsWith()` | |
| `.replace()` | `.replace()` | |
| `.split()` | `.split()` | |
| `.join()` | `.join()` | |
| `.map()` | `.map()` | |
| `.filter()` | `.filter()` | |
| `.find()` | `.find()` | |
| `.iter()` | No-op | |
| `.collect()` | No-op | |
| `.enumerate()` | Optimized `for` loop with index counter | |
| `.keys()` | `Object.keys(receiver)` | |
| `.get()` | IIFE: `.get()` or `obj[key]` | Universal for Map/Object |
| `.insert()` | IIFE: `.splice()` or `obj[key] = val` | Universal for Array/Object |
| `.remove()` | IIFE: `.splice()` or `delete obj[key]` | Universal for Array/Object |
| `.is_some()` | `!== null && !== undefined` | IIFE for function call receivers |
| `.is_none()` | `=== null \|\| === undefined` | IIFE for function call receivers |
| `.unwrap()` | No-op (returns receiver) | |
| Other methods | Pass-through (`.methodName()`) | Kept as-is |

---

## 5. Macro Transpilations

| Rust Macro | JavaScript Output | Notes |
|------------|------------------|-------|
| `println!("...")` | `console.log(...)` | With format string support |
| `print!("...")` | `console.log(...)` | |
| `eprintln!("...")` | `console.error(...)` | |
| `eprint!("...")` | `console.error(...)` | |
| `format!("...", args)` | Template literal | `{}` becomes `${arg}`, `{:?}` becomes `${debug_repr(arg)}` |
| `vec![a, b, c]` | `[a, b, c]` | |
| `vec![val; count]` | `Array.from({length: count}, () => val)` | |
| `panic!()` | Not supported | Not transpiled |
| `todo!()` | Not supported | |
| `assert!()` | Not supported | |
| `dbg!()` | Not supported | |

---

## 6. Pattern Matching

| Pattern Type | Status | Notes |
|--------------|--------|-------|
| Literal patterns (`1`, `"str"`, `true`) | Supported | Uses `===` comparison |
| Wildcard (`_`) | Supported | Generates `true` condition (always matches) |
| Variable binding (`x`) | Supported | Creates `const x = _match_value` |
| `None` | Supported | Checks `=== null \|\| === undefined` |
| `Some(x)` | Supported | Checks `!== null && !== undefined` with binding |
| `Some((a, b))` tuple inside Some | Supported | Array index access |
| Enum unit variants | Supported | String comparison on match value |
| Enum tuple variants (`Variant(x)`) | Supported | Checks `.type === 'Variant'`, binds `.value0` etc. |
| Enum struct variants (`Variant { field }`) | Supported | Checks `.type`, binds named fields |
| `Ok(x)` / `Err(e)` | Supported | Checks `.ok !== undefined` / `.error !== undefined` |
| Tuple patterns (`(a, b)`) | Supported | Array index access with combined conditions |
| Nested `Some`/`None` in tuples | Supported | |
| Or patterns (`A \| B`) | Not supported | |
| Guard clauses (`if cond`) | Not supported | |
| Range patterns (`1..=5`) | Not supported | |
| Ref patterns (`ref x`) | Not supported | |
| Slice patterns (`[a, b, ..]`) | Not supported | |

---

## 7. Browser/Web APIs (DOM API Mock Coverage)

### 7.1 DOM Manipulation

| API | Status | Notes |
|-----|--------|-------|
| `document.getElementById()` | Supported | Mock implementation |
| `document.getElementsByTagName()` | Supported | |
| `document.getElementsByClassName()` | Supported | |
| `document.getElementsByName()` | Supported | |
| `document.querySelector()` | Supported | |
| `document.querySelectorAll()` | Supported | |
| `document.createElement()` | Supported | |
| `document.createTextNode()` | Supported | |
| `document.createDocumentFragment()` | Supported | |
| `document.adoptNode()` | Supported | |
| `document.importNode()` | Supported | |
| `document.write()` / `writeln()` | Supported | |
| `document.open()` / `close()` | Supported | |
| `document.body` / `head` / `documentElement` | Supported | As methods |
| `document.title` | Supported | |
| `document.URL` | Supported | |
| Element properties (`id`, `className`, `innerHTML`, etc.) | Supported | |
| `element.classList` | Supported | Full `DOMTokenList` with `add`, `remove`, `toggle`, `contains`, `replace` |
| `element.getAttribute()` / `setAttribute()` | Supported | |
| `element.removeAttribute()` / `hasAttribute()` | Supported | |
| `element.addEventListener()` | Supported | |
| `element.removeEventListener()` | Supported | |
| `element.appendChild()` / `removeChild()` | Supported | |
| `element.insertBefore()` / `replaceChild()` | Supported | |
| `element.insertAdjacentHTML()` | Supported | |
| `element.cloneNode()` | Supported | |
| `element.querySelector()` / `querySelectorAll()` | Supported | |
| `element.closest()` | Supported | |
| `element.matches()` | Supported | |
| `element.focus()` / `blur()` / `click()` | Supported | |
| `element.scrollIntoView()` | Supported | |
| `element.getBoundingClientRect()` | Supported | Returns `DOMRect` |
| `element.style` (via CSSStyleDeclaration) | Supported | 18 CSS properties |
| `element.dataset` | Not supported | |
| `element.children` / `childNodes` | Not supported | |
| `element.parentElement` / `parentNode` | Not supported | |
| `element.nextSibling` / `previousSibling` | Not supported | |
| `element.firstChild` / `lastChild` | Not supported | |
| `element.offsetWidth` / `offsetHeight` etc. | Not supported | |
| `element.shadowRoot` / `attachShadow()` | Not supported | |
| `element.animate()` | Not supported | |
| `MutationObserver` | Not supported | |
| `IntersectionObserver` | Not supported | |
| `ResizeObserver` | Not supported | |
| `TreeWalker` / `NodeIterator` | Not supported | |

### 7.2 Events

| API | Status | Notes |
|-----|--------|-------|
| `Event` object | Supported | With `type`, `bubbles`, `cancelable`, `target`, `currentTarget`, `timeStamp` |
| `event.preventDefault()` | Supported | |
| `event.stopPropagation()` | Supported | |
| `event.stopImmediatePropagation()` | Supported | |
| `MessageEvent` | Supported | With `data` property |
| `CustomEvent` | Not supported | |
| `KeyboardEvent` | Not supported | |
| `MouseEvent` | Not supported | |
| `TouchEvent` | Not supported | |
| `FocusEvent` | Not supported | |
| `InputEvent` | Not supported | |
| `DragEvent` | Not supported | |
| `WheelEvent` | Not supported | |
| `PointerEvent` | Not supported | |
| `ClipboardEvent` | Not supported | |
| `EventTarget` interface | Partial | `addEventListener`/`removeEventListener` on Element and Window |

### 7.3 Window and Navigation

| API | Status | Notes |
|-----|--------|-------|
| `window.alert()` | Supported | |
| `window.confirm()` | Supported | |
| `window.prompt()` | Supported | |
| `window.open()` / `close()` | Supported | |
| `window.print()` | Supported | |
| `window.focus()` / `blur()` | Supported | |
| `window.scrollTo()` / `scrollBy()` | Supported | |
| `window.innerWidth` / `innerHeight` | Supported | |
| `window.outerWidth` / `outerHeight` | Supported | |
| `window.pageXOffset` / `pageYOffset` | Supported | |
| `window.getComputedStyle()` | Supported | |
| `window.addEventListener()` | Supported | |
| `window.matchMedia()` | Not supported | |
| `window.postMessage()` | Not supported | |
| `window.performance` | Not supported | |
| `window.crypto` | Not supported | |
| `window.screen` | Not supported | |
| `window.devicePixelRatio` | Not supported | |
| `location.href` / `protocol` / `host` etc. | Supported | Full Location interface |
| `location.reload()` / `assign()` / `replace()` | Supported | |
| `history.back()` / `forward()` / `go()` | Supported | |
| `history.pushState()` / `replaceState()` | Supported | |
| `navigator.userAgent` / `language` / `platform` | Supported | |
| `navigator.mediaDevices` | Supported | See WebRTC section |

### 7.4 Timers

| API | Status | Notes |
|-----|--------|-------|
| `setTimeout()` | Supported | With closure support |
| `setInterval()` | Supported | With closure support |
| `clearTimeout()` | Supported | |
| `clearInterval()` | Supported | |
| `requestAnimationFrame()` | Supported | With closure support |
| `cancelAnimationFrame()` | Supported | |
| `queueMicrotask()` | Not supported | |
| `requestIdleCallback()` | Not supported | |

### 7.5 Network and Data

| API | Status | Notes |
|-----|--------|-------|
| `XMLHttpRequest` | Supported | Full lifecycle mock: `open`, `send`, `setRequestHeader`, `abort`, etc. |
| `XMLHttpRequestUpload` | Supported | With event listeners |
| `ProgressEvent` | Supported | |
| `fetch()` | Not supported | Major gap; `XMLHttpRequest` is the only HTTP option |
| `Headers` | Not supported | |
| `Request` / `Response` | Not supported | |
| `URL` / `URLSearchParams` | Not supported | |
| `FormData` | Not supported | |
| `AbortController` / `AbortSignal` | Not supported | |
| `Blob` / `File` / `FileReader` | Not supported | |
| `ReadableStream` / `WritableStream` | Not supported | |
| `TextEncoder` / `TextDecoder` | Not supported | |

### 7.6 WebSocket

| API | Status | Notes |
|-----|--------|-------|
| `WebSocket` constructor | Supported | |
| `WebSocket.send()` | Supported | |
| `WebSocket.close()` | Supported | |
| `WebSocket.addEventListener()` | Supported | With `MessageEvent` callback |
| `WebSocket.readyState` | Not supported | |
| `WebSocket.bufferedAmount` | Not supported | |
| `WebSocket.protocol` | Not supported | |

### 7.7 Storage

| API | Status | Notes |
|-----|--------|-------|
| `localStorage.setItem()` | Supported | Full implementation |
| `localStorage.getItem()` | Supported | |
| `localStorage.removeItem()` | Supported | |
| `localStorage.clear()` | Supported | |
| `localStorage.key()` | Supported | |
| `localStorage.length` | Supported | |
| `localStorage.setJSON()` | Supported | Extension: JSON serialize/deserialize helpers |
| `localStorage.getJSON()` | Supported | Extension |
| `sessionStorage` | Supported | Delegates to localStorage mock |
| `IndexedDB` | Not supported | |
| `Cache` / `CacheStorage` | Not supported | |
| Cookies (`document.cookie`) | Not supported | |

### 7.8 WebRTC

| API | Status | Notes |
|-----|--------|-------|
| `RTCPeerConnection` | Supported | Core WebRTC with ICE, SDP, track management |
| `RTCPeerConnection.createOffer()` | Supported | Returns `Promise<RTCSessionDescription>` |
| `RTCPeerConnection.createAnswer()` | Supported | |
| `RTCPeerConnection.setLocalDescription()` | Supported | |
| `RTCPeerConnection.setRemoteDescription()` | Supported | |
| `RTCPeerConnection.addIceCandidate()` | Supported | |
| `RTCPeerConnection.addTrack()` | Supported | |
| `RTCPeerConnection.removeTrack()` | Supported | |
| `RTCPeerConnection.getSenders()` | Supported | |
| `RTCPeerConnection.createDataChannel()` | Supported | |
| `RTCPeerConnection.addEventListener()` | Supported | With unified event type |
| `RTCPeerConnection.close()` | Supported | |
| `RTCSessionDescription` | Supported | |
| `RTCIceCandidate` | Supported | With `RTCIceCandidateInit` |
| `RTCConfiguration` | Supported | With `#[js_type]` export |
| `RTCIceServer` | Supported | With `#[js_type]` export |
| `RTCDataChannel` | Supported | With `send`, `close`, `addEventListener` |
| `RTCRtpSender` | Supported | With `replaceTrack`, `getStats` |
| `RTCRtpReceiver` | Supported | |
| `RTCRtpTransceiver` | Supported | |
| `RTCStatsReport` | Supported | Basic interface |
| `MediaStream` | Supported | With track management and events |
| `MediaStreamTrack` | Supported | With `stop`, `clone`, events |
| `MediaDevices.getUserMedia()` | Supported | Returns `Promise<MediaStream>` |
| `MediaDevices.getDisplayMedia()` | Supported | |
| `MediaDevices.enumerateDevices()` | Supported | |
| `MediaStreamConstraints` | Supported | |
| WebRTC event types | Supported | `RTCPeerConnectionIceEvent`, `RTCTrackEvent`, `RTCDataChannelEvent` |

### 7.9 Not Covered Web APIs

| API Category | Status | Notes |
|--------------|--------|-------|
| Canvas 2D / WebGL / WebGPU | Not supported | |
| Web Workers / SharedWorker | Not supported | |
| Service Workers | Not supported | |
| Web Audio API | Not supported | |
| Web Animations API | Not supported | |
| Clipboard API | Not supported | |
| Notifications API | Not supported | |
| Geolocation API | Not supported | |
| Fullscreen API | Not supported | |
| Gamepad API | Not supported | |
| Battery API | Not supported | |
| Vibration API | Not supported | |
| Web Speech API | Not supported | |
| Web Serial / Web Bluetooth / Web USB | Not supported | |
| Web NFC | Not supported | |
| Payment Request API | Not supported | |
| Web Share API | Not supported | |
| File System Access API | Not supported | |
| Broadcast Channel | Not supported | |
| Web Locks API | Not supported | |
| Web Crypto API | Not supported | |
| Performance API / PerformanceObserver | Not supported | |
| Reporting API | Not supported | |

### 7.10 Global Functions

| Function | Status | Notes |
|----------|--------|-------|
| `alert()` | Supported | |
| `confirm()` | Supported | |
| `prompt()` | Supported | |
| `parseFloat()` | Supported | |
| `parseInt()` | Supported | |
| `isNaN()` | Supported | |
| `isFinite()` | Supported | |
| `encodeURIComponent()` | Supported | Mock only |
| `decodeURIComponent()` | Supported | Mock only |
| `Number()` (global function) | Supported | For DOM API usage |
| `JSON.parse()` | Not supported | No transpilation mapping |
| `JSON.stringify()` | Not supported | |
| `eval()` | Not supported | |
| `structuredClone()` | Not supported | |
| `atob()` / `btoa()` | Not supported | |
| `encodeURI()` / `decodeURI()` | Not supported | |

---

## 8. Console API

| Method | Status | Notes |
|--------|--------|-------|
| `console.log()` | Supported | From `println!` macro |
| `console.error()` | Supported | From `eprintln!` macro |
| `console.warn()` | Supported | Mock available |
| `console.info()` | Supported | Mock available |
| `console.debug()` | Supported | Mock available |
| `console.trace()` | Supported | Mock available |
| `console.group()` / `groupEnd()` | Supported | Mock available |
| `console.time()` / `timeEnd()` | Supported | Mock available |
| `console.clear()` | Supported | Mock available |
| `console.count()` / `countReset()` | Supported | Mock available |
| `console.table()` | Supported | Mock available |
| `console.dir()` | Not supported | |
| `console.assert()` | Not supported | |
| `console.profile()` | Not supported | |

---

## 9. Summary of Key Gaps

### High Priority (Common JS patterns not yet covered)

1. **`fetch()` API** -- The modern replacement for XMLHttpRequest is not available. This is arguably the most impactful missing Web API.

2. **`try`/`catch`/`finally`** -- Rust's `?` operator is transpiled to an IIFE with error-field checking, but there is no way to write actual try/catch blocks in the generated JS. Rust's `panic!` is not transpiled at all.

3. **JS Modules (`import`/`export`)** -- The transpiler uses `linkme::distributed_slice` for code collection. There is no ES module generation. This limits interop with existing JS ecosystems.

4. **`Map` and `Set`** -- `HashMap` transpiles to plain objects `{}` rather than `new Map()`. This means Map-specific features (non-string keys, iteration order guarantees, `.size`) are lost.

5. **Optional chaining (`?.`) and nullish coalescing (`??`)** -- These modern JS operators have no Rust mapping. The transpiler could potentially generate them from Rust's `?` and `unwrap_or` patterns.

6. **Spread/rest operators (`...`)** -- No support for rest parameters, spread in function calls, or spread in object/array literals.

7. **`continue` statement** -- Recognized but transpiled to `undefined` instead of an actual `continue` statement. This would cause incorrect behavior in loops.

8. **Default function parameters** -- No way to express default parameter values in the generated JS.

### Medium Priority

9. **Generators and iterators** -- No support for `function*`, `yield`, or custom iterators.

10. **Class inheritance** -- No `extends`, `super`, or trait-to-interface mapping.

11. **Getters/setters** -- Neither in class definitions nor object literals.

12. **Private class fields (`#field`)** -- Not generated; all fields are public in the output.

13. **Regular expressions** -- No transpilation from Rust regex crate to JS regex.

14. **`Promise.all/race/allSettled/any`** -- Only basic `.then()` and `.catch()` are mocked.

15. **`switch` statement** -- `match` expressions generate if/else chains rather than switch. This works but is less idiomatic.

16. **Labeled loops/break** -- Rust labeled loops (`'label: loop`) cannot be transpiled.

17. **Match guard clauses** -- Rust `arm if condition => ...` is not supported.

18. **Or-patterns in match** -- Rust `A | B => ...` is not supported.

### Low Priority

19. **`BigInt`** -- No Rust equivalent.
20. **`Symbol`** -- No Rust equivalent.
21. **`Proxy`/`Reflect`** -- Very specialized.
22. **`WeakMap`/`WeakSet`/`WeakRef`** -- Niche use cases.
23. **Canvas/WebGL/WebGPU** -- Specialized graphics APIs.
24. **Web Workers / Service Workers** -- Specialized APIs.
25. **Most newer Web APIs** (Clipboard, Notifications, Geolocation, etc.) -- Can be added as needed.

---

## 10. Architecture Notes for Future Work

### Adding new method mappings
New Rust-to-JS method mappings are added in the `handle_method_call()` function in `mojes-mojo/src/lib.rs` (around line 2126). Each mapping is a match arm on the method name string.

### Adding new expression types
New Rust expression types are handled in `rust_expr_to_js_with_action_and_state()` (around line 1514). The function matches on `syn::Expr` variants.

### Adding new DOM APIs
New browser API mocks are added in `mojes-dom-api/src/lib.rs`. Each API is implemented as a Rust struct with methods that print debug output. The struct is exposed as a `pub static` global.

### Adding new proc macros
New proc macros go in `mojes-derive/src/lib.rs`. The existing macros (`#[to_js]`, `#[js_type]`, `#[js_object]`) can serve as templates.

### Key design pattern: IIFE wrapping
The transpiler makes heavy use of IIFEs (Immediately Invoked Function Expressions) to handle Rust's "everything is an expression" semantics. Block expressions, match expressions, if-as-expression, and various method transpilations all use this pattern. This is correct but generates verbose output.

### Key design pattern: Universal method dispatch
Collection methods like `len()`, `get()`, `insert()`, `remove()`, and `contains_key()` use IIFE-based runtime type checking to handle both arrays/objects and Map objects. This provides correctness at the cost of runtime overhead.
