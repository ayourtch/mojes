# mojes

Write browser clients in Rust; ship them as plain JavaScript.

mojes is an **experimental Rust-to-JavaScript source-to-source transpiler**.
Your client code is ordinary Rust — it type-checks, borrow-checks, and can be
unit-tested with `cargo test` — and procedural macros transpile it to
JavaScript **at macro-expansion time**, while your crate compiles. The
generated JS fragments are collected across the crate with
[`linkme`](https://crates.io/crates/linkme) distributed slices, so at runtime
your (server-side) Rust program can simply join them into a single `client.js`
and serve it to the browser.

**There is no WebAssembly involved.** This is not `wasm-bindgen`, not
Emscripten, and nothing runs a Rust runtime in the browser. The Rust AST
(parsed with `syn`) is translated into a JavaScript AST (built and printed
with [SWC](https://swc.rs/)), and the browser executes hand-off-quality plain
JavaScript that calls the real DOM/WebRTC/WebSocket APIs directly.

## Status

Experimental. The transpiler covers a useful subset of Rust (enough to write
real interactive browser clients — see
[the sample project](#example-project-mojes-sample)), but plenty of Rust does
not transpile, some things transpile with sharp edges (see
[Gotchas](#gotchas)), and the APIs may change without notice. Version 0.1.0,
not published to crates.io.

## Workspace layout

| Crate | What it is |
|---|---|
| [`mojes`](mojes/) | Facade crate: re-exports the macros, the transpiler, and the DOM API (as `mojes::dom`), plus `linkme` and a `prelude`. |
| [`mojes-derive`](mojes-derive/) | The proc macros: `#[to_js]`, `#[js_type]`, `#[js_object]` (and `#[impl_to_js]`). |
| [`mojes-mojo`](mojes-mojo/) | The transpilation engine: `syn` AST in, SWC JavaScript AST out. All the language-mapping logic lives here. |
| [`mojes-dom-api`](mojes-dom-api/) | Rust stubs for browser APIs (DOM, console, WebRTC, WebSocket, storage, timers, …) so client code type-checks and can run in tests. |
| [`mojes-file`](mojes-file/) | A small CLI that transpiles a `.rs` file to JS on stdout and can optionally execute the result with the [Boa](https://github.com/boa-dev/boa) JS engine. Handy for experiments. |

## How it works

1. You annotate functions, structs/enums, and `impl` blocks with the mojes
   macros.
2. At macro-expansion time, each macro transpiles the item to a JavaScript
   string and emits it into a `linkme` distributed slice named `crate::JS`
   (alongside the untouched Rust item, which still compiles normally).
3. At runtime you collect the slice — `JS.join("\n\n")` — and serve the result
   as your page's script. There is no separate build step, no bundler, and no
   ES modules: the output is one flat script.

Your crate needs a `JS` slice at the crate root. Either declare one yourself:

```rust
#[mojes::distributed_slice]
pub static JS: [&str];
```

or glob-import the DOM API at the crate root (`use mojes::dom::*;`), which
brings in the slice declared by `mojes-dom-api`.

## A minimal example

```rust
use mojes::dom::*;                     // browser API stubs + the JS slice
use mojes::to_js;                      // one attribute for everything

#[to_js]                               // on a struct → a JS class with toJSON/fromJSON
struct Counter {
    count: i32,
    label: String,
}

#[to_js]                               // on an impl → methods on Counter.prototype
impl Counter {
    fn new(label: String) -> Counter {
        Counter { count: 0, label }    // fields in declaration order!
    }

    fn increment(&mut self) {
        self.count += 1;
        if let Some(mut el) = document.getElementById("counter") {
            el.textContent = format!("{}: {}", self.label, self.count);
        }
    }
}

#[to_js]                               // on a function → a plain JS function
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Everything above, as JavaScript — serve this from your web server.
pub fn client_js() -> String {
    JS.join("\n\n")
}
```

The generated JavaScript (lightly reformatted):

```javascript
class Counter {
    constructor(count, label) {
        this.count = count;
        this.label = label;
    }
    toJSON() { return { count: this.count, label: this.label }; }
    static fromJSON(json) { return new Counter(json.count, json.label); }
}

Counter.new = function(label) {
    return new Counter(0, label);
};
Counter.prototype.increment = function() {
    this.count += 1;
    const el = document.getElementById("counter");
    {
        const temp_0 = document.getElementById("counter");
        if (temp_0 !== null && temp_0 !== undefined) {
            const el = temp_0;
            el.textContent = `${this.label}: ${this.count}`;
        }
    }
};

function greet(name) {
    // Type validation for name
    if (typeof name !== 'string') {
        throw new TypeError('Expected name to be of type string, got ' + typeof name);
    }
    return `Hello, ${name}!`;
}
```

Note how Rust idioms map: `Option` becomes `null`/`undefined` checks,
`format!` becomes a template literal, struct construction becomes a
positional `new Counter(...)` call, and `#[to_js]` adds runtime type checks
for primitive parameters. `Some(...)`/`None` disappear — an `Option<T>` is
just a nullable value in JS.

### The macros

**`#[to_js]` works on every supported item kind** and dispatches on what it
is applied to — it is the only attribute you need to remember:

- **on a function** — transpiles it to a plain JS function. Async functions
  become `async function`s; a failed transpilation is reported as a normal
  `rustc` compile error pointing at the function, with the reason.
- **on a struct or enum** — generates the JS class (positional constructor,
  `toJSON`/`fromJSON`) and, for enums, JSON helpers on both the Rust and JS
  sides so the two ends can exchange the same wire format via `serde_json`.
- **on an inherent `impl` block** — transpiles every method. Associated
  functions (no `self`) become statics (`Type.new = ...`), methods become
  `Type.prototype.method = ...`. Call sites of `Type::new(...)` dispatch to
  the custom static `Type.new` when one exists, falling back to the
  positional `new Type(...)` constructor otherwise.

`#[js_type]` (structs/enums) and `#[js_object]` (impl blocks) remain as
explicit spellings of the same transpilations.

## The browser API stubs (`mojes-dom-api`)

The browser APIs are **compile-time type-checking stubs**. `mojes-dom-api`
defines Rust structs and globals — `document`, `window`, `console`,
`localStorage`, `Element`, `WebSocket`, `RTCPeerConnection`, `MediaStream`,
`XMLHttpRequest`, timers, `alert`/`confirm`/`prompt`, a `JSON` global, a
`Promise<T>` type (which implements `Future`, so `.await` works), and so on —
whose method names are exactly the JavaScript ones (`getElementById`,
`addEventListener`, `setLocalDescription`, …).

At transpile time, method calls **pass straight through by name**: the
transpiler does not know or care about the stubs; `document.getElementById(x)`
in Rust simply becomes `document.getElementById(x)` in JS, resolved against
the real browser objects at runtime. The stubs exist so that:

- your client code type-checks under `rustc` (typos and wrong argument types
  are compile errors);
- Rust-side tests can call the same code paths (the stubs are lightweight mock
  implementations).

Coverage is driven by what real client code has needed so far; see
[`JS_COVERAGE_REPORT.md`](JS_COVERAGE_REPORT.md) for the detailed API-by-API
status (notably: no `fetch()` yet — use `XMLHttpRequest` or `WebSocket`).

## What transpiles (and what doesn't)

Supported (exercised by the test suite under `mojes-mojo/tests/` and
`mojes/tests/`):

- expressions, arithmetic/logic, `format!`/`println!`/`eprintln!` (→ template
  literals, `console.log`/`console.error`), string and collection methods
  (`push`, `len`, `contains`, `insert`, `remove`, `contains_key`, `keys`, …);
- `if`/`else` (including as expressions), `while`, `loop`, `for` (including
  `for (k, v) in map` via a universal `entries()` shim), early `return`;
- `match` with tuple patterns, struct/enum destructuring, or-patterns
  (`1 | 2 | 3 =>`), wildcards; `if let`;
  `let else`-style flows via `Option` checks;
- closures (including wildcard `_` parameters), passing closures as callbacks;
- `async fn` / `.await`, `Promise` chaining (`then`/`catch`), `?` on async
  results (→ `try`/`catch`-style error propagation);
- structs, enums (unit/tuple/struct variants) with JSON round-tripping,
  inherent impls, `self` methods;
- `HashMap`/`Vec`/`Option`/`Result` in their common usage patterns;
- raw identifiers (`r#type`), variable shadowing (automatic renaming),
  per-arm match scoping.

Not supported (see [`JS_COVERAGE_REPORT.md`](JS_COVERAGE_REPORT.md) §9 and
[`BLOCKS.md`](BLOCKS.md) for details): traits/generics in transpiled code,
match guards (`arm if cond =>`), labeled loops,
`continue` (transpiles incorrectly — avoid), iterators/generators,
class inheritance, spread/rest, regexes, `fetch`, ES modules. `match`
compiles to `if`/`else` chains, and `HashMap` becomes a plain object, not a
`Map`.

### Gotchas

Learned the hard way while building a real client — the transpiled Rust
compiles fine but the JS misbehaves:

- **No `match` around `.await`.** Match arms are transpiled into non-async
  IIFEs, so an `.await` inside a match arm breaks. On async paths use
  if-chains with `unwrap_or`/`is_some` instead.
- **Struct literals must list fields in declaration order.** The generated JS
  constructor is positional; `Counter { label, count }` would silently swap
  the arguments.
- **Use `"".to_string()`, not `String::new()`.** The latter transpiles to
  `new String()`, a boxed object rather than a string primitive, and `===`
  comparisons against it fail.
- Conditional (`if`/`else`) expressions used directly inside `format!`
  arguments have historically produced `undefined` — see
  [`BLOCKS.md`](BLOCKS.md). Bind them to a `let` first.

Since the output is generated at compile time, a cheap end-to-end sanity check
is to dump the JS and parse it with node:

```sh
cargo run -p your-client --bin dump-js > /tmp/client.js && node --check /tmp/client.js
```

## Trying it without writing a crate

`mojes-file` transpiles a bare `.rs` file (no macros needed) and can run the
result under the Boa engine:

```sh
cargo run -p mojes-file -- input.rs            # JS to stdout
cargo run -p mojes-file -- input.rs -o out.js  # JS to a file
cargo run -p mojes-file -- input.rs --run main # transpile and execute with Boa
```

## Running the tests

```sh
cargo test                    # whole workspace
cargo test -p mojes-mojo      # transpiler engine tests
cargo test -p mojes-dom-api   # browser-stub tests
```

The test suite is large and doubles as the supported-feature list: most tests
transpile a Rust snippet and then **execute the generated JavaScript with the
Boa engine**, asserting on its behavior — not just on the emitted text.

## Example project: mojes-sample

[`mojes-sample`](https://github.com/ayourtch/mojes-sample) is an interactive
tour of the transpiler: a small web server whose entire browser side — DOM
manipulation, events, timers, canvas animation, form handling, storage, XHR —
is written in Rust and transpiled by mojes. Clone it side by side with this
repository, `cargo run`, and open <http://localhost:3000/> with the dev tools
open; "view source" shows the generated JavaScript next to the Rust in
`src/main.rs`. It is the best reference for what mojes can do today and for
the patterns (and gotchas) of writing real clients.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
