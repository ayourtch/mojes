use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Test event type for closure parameters
#[js_type]
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub event_type: String,
    pub data: String,
}

#[js_object]
impl TestEvent {
    pub fn new(event_type: String, data: String) -> Self {
        Self { event_type, data }
    }
}

// Test function that takes a closure
#[to_js]
fn test_func_with_closure<F>(callback: F) 
where 
    F: Fn(TestEvent)
{
    let event = TestEvent::new("test".to_string(), "sample data".to_string());
    callback(event);
}

// Test closures without explicit parameter types
#[to_js]
fn test_closure_without_types() {
    println!("Testing closure without explicit parameter types...");
    
    // Closure without explicit parameter type
    test_func_with_closure(|e| {
        println!("Received event without type annotation: {:?}", e);
        println!("Event type: {}", e.event_type);
        println!("Event data: {}", e.data);
    });
}

// Test closures with explicit parameter types  
#[to_js]
fn test_closure_with_types() {
    println!("Testing closure with explicit parameter types...");
    
    // Closure with explicit parameter type
    test_func_with_closure(|e: TestEvent| {
        println!("Received event with type annotation: {:?}", e);
        println!("Event type: {}", e.event_type);
        println!("Event data: {}", e.data);
    });
}

// Test multiple parameter closures
#[to_js]
fn test_multi_param_closures() {
    println!("Testing multi-parameter closures...");
    
    let process_data = |name, age, active| {
        println!("Name: {}, Age: {}, Active: {}", name, age, active);
    };
    
    let process_data_typed = |name: String, age: i32, active: bool| {
        println!("Typed - Name: {}, Age: {}, Active: {}", name, age, active);
    };
    
    process_data("Alice".to_string(), 30, true);
    process_data_typed("Bob".to_string(), 25, false);
}

// Test closure return values
#[to_js]
fn test_closure_return_values() {
    println!("Testing closure return values...");
    
    // Closure without type annotation that returns a value
    let transform_untyped = |data| {
        format!("Transformed: {}", data)
    };
    
    // Closure with type annotation that returns a value
    let transform_typed = |data: String| -> String {
        format!("Typed Transform: {}", data)
    };
    
    let result1 = transform_untyped("hello".to_string());
    let result2 = transform_typed("world".to_string());
    
    println!("Result 1: {}", result1);
    println!("Result 2: {}", result2);
}

// Test nested closures
#[to_js]
fn test_nested_closures() {
    println!("Testing nested closures...");
    
    let outer_closure = |x| {
        let inner_closure = |y| {
            x + y
        };
        
        let inner_closure_typed = |y: i32| -> i32 {
            x + y + 10
        };
        
        let result1 = inner_closure(5);
        let result2 = inner_closure_typed(5);
        
        println!("Inner result 1: {}", result1);
        println!("Inner result 2: {}", result2);
    };
    
    outer_closure(10);
}

// Test closures with complex types
#[to_js]  
fn test_complex_type_closures() {
    println!("Testing closures with complex types...");
    
    let events = vec![
        TestEvent::new("click".to_string(), "button1".to_string()),
        TestEvent::new("hover".to_string(), "button2".to_string()),
    ];
    
    // Without type annotation
    events.iter().for_each(|event| {
        println!("Processing event: {:?}", event);
    });
    
    // With type annotation
    events.iter().for_each(|event: &TestEvent| {
        println!("Typed processing: {} -> {}", event.event_type, event.data);
    });
}

#[test]
fn test_closure_parameter_transpilation() {
    println!("=== CLOSURE PARAMETER TYPE TRANSPILATION TEST ===");
    
    println!("Generated JavaScript functions:");
    for (i, js_code) in JS.iter().enumerate() {
        println!("--- Function {} ---", i + 1);
        println!("{}", js_code);
        println!();
    }
    
    println!("=== ANALYSIS ===");
    println!("Look for the following patterns in the JavaScript:");
    println!("1. Closures without type annotations: |e| => function(e)");
    println!("2. Closures with type annotations: |e: TestEvent| => function(e) or missing");
    println!("3. Multi-parameter closures with/without types");
    println!("4. Return type annotations: |x| -> String => function(x)");
    println!("5. Complex type annotations: |event: &TestEvent| => function(event)");
    println!();
    println!("Expected behavior:");
    println!("- Untyped closures should transpile to proper JavaScript functions");
    println!("- Typed closures should also transpile but parameter types should be ignored");
    println!("- Function body should be preserved in both cases");
    println!("- Return types should be ignored but function should still return values");
    
    // Basic test - should have generated some JavaScript
    assert!(!JS.is_empty(), "No JavaScript code was generated");
    
    // Look for closure patterns in the generated JS
    let combined_js = JS.join("\n");
    
    // Should contain function definitions (closures become functions in JS)  
    assert!(combined_js.contains("function"), 
           "Generated JavaScript should contain function definitions for closures");
           
    println!("\n✅ Test completed - check the generated JavaScript above for closure transpilation patterns");
}

/*
This test verifies closure parameter type handling by comparing:

1. test_closure_without_types() - uses |e| { ... }
2. test_closure_with_types() - uses |e: TestEvent| { ... }  
3. test_multi_param_closures() - tests multiple parameters with/without types
4. test_closure_return_values() - tests return type annotations
5. test_nested_closures() - tests nested closure scenarios
6. test_complex_type_closures() - tests complex reference types

The suspected issue:
- |e| { ... } should transpile to function(e) { ... }
- |e: TestEvent| { ... } might transpile to function() { ... } (missing parameter)

This test will show the actual generated JavaScript so we can verify
if parameter types are causing the parameters to be dropped during transpilation.
*/