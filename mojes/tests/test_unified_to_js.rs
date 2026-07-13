// The unified interface: #[to_js] works on every supported item kind -
// functions, structs, enums, and impl blocks - dispatching to the same
// transpilation paths as #[js_type] / #[js_object]. This test uses ONLY
// #[to_js] and verifies both the emitted JS and its execution under Boa.

use linkme::distributed_slice;
use mojes_derive::to_js;

#[distributed_slice]
static JS: [&str] = [..];

#[to_js]
#[derive(Debug, Clone)]
struct Counter {
    label: String,
    count: i32,
}

#[to_js]
#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Idle,
    Running,
}

#[to_js]
impl Counter {
    fn new(label: String) -> Self {
        Self { label, count: 0 }
    }

    fn bump(&mut self) -> i32 {
        self.count += 1;
        self.count
    }

    fn describe(&self) -> String {
        format!("{}: {}", self.label, self.count)
    }
}

#[to_js]
pub fn unified_entry() -> String {
    let mut c = Counter::new("clicks".to_string());
    c.bump();
    c.bump();
    let mode = Mode::Running;
    if mode == Mode::Running {
        c.bump();
    }
    c.describe()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, JsResult, JsValue, Source};

    fn eval_js(code: &str) -> JsResult<String> {
        let mut context = Context::default();
        let console_log = |_this: &JsValue,
                           args: &[JsValue],
                           ctx: &mut Context|
         -> JsResult<JsValue> {
            let message = args
                .iter()
                .map(|arg| arg.to_string(ctx).unwrap().to_std_string().unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            println!("JS Console: {}", message);
            Ok(JsValue::undefined())
        };
        let console_obj = boa_engine::object::ObjectInitializer::new(&mut context)
            .function(
                boa_engine::native_function::NativeFunction::from_fn_ptr(console_log),
                "log",
                0,
            )
            .build();
        context
            .register_global_property(
                "console",
                console_obj,
                boa_engine::property::Attribute::all(),
            )
            .unwrap();
        let result = context.eval(Source::from_bytes(code))?;
        Ok(result
            .to_string(&mut context)
            .unwrap()
            .to_std_string()
            .unwrap())
    }

    #[test]
    fn to_js_dispatches_on_item_kind() {
        let full_js = JS.join("\n");

        // Struct -> a JS class with a positional constructor.
        assert!(full_js.contains("class Counter"), "no class for struct");
        // Enum -> a JS enum object.
        assert!(full_js.contains("Mode"), "no JS for enum");
        // Impl -> methods attached to the class.
        assert!(full_js.contains("bump"), "no method from impl block");
        assert!(full_js.contains("describe"), "no method from impl block");
        // Function -> a plain JS function.
        assert!(
            full_js.contains("function unified_entry()"),
            "no function output"
        );

        // The Rust side still runs as ordinary Rust.
        assert_eq!(unified_entry(), "clicks: 3");

        // And the transpiled JS computes the same result.
        let test_js = format!("{}\nunified_entry();", full_js);
        let result = eval_js(&test_js).expect("JS execution failed");
        assert_eq!(result, "clicks: 3", "JS and Rust results differ");
    }
}
