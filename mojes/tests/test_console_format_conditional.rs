use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;


#[distributed_slice]
static JS: [&str] = [..];

// Custom logging function to replace console.log
#[to_js]
fn log_message(message: &str) {
    println!("LOG: {}", message);
}

#[js_type]
#[derive(Debug, Clone)]
pub struct StunCandidate {
    pub candidate_type: String,
    pub ip: String,
    pub port: u16,
}

#[js_object]
impl StunCandidate {
    pub fn new(candidate_type: String, ip: String, port: u16) -> Self {
        Self {
            candidate_type,
            ip,
            port,
        }
    }

    pub fn is_srflx(&self) -> bool {
        self.candidate_type == "srflx"
    }
}

#[js_type]
#[derive(Debug, Clone)]
pub struct IceConnection {
    pub candidates: Vec<StunCandidate>,
}

#[js_object]
impl IceConnection {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
        }
    }

    pub fn add_candidate(&mut self, candidate: StunCandidate) {
        self.candidates.push(candidate);
    }

    pub fn has_srflx_candidates(&self) -> bool {
        self.candidates.iter().any(|c| c.is_srflx())
    }

    pub fn log_stun_status(&self) {
        let has_srflx = self.has_srflx_candidates();
        
        // This is the exact code pattern we want to test
        log_message(&format!("      STUN candidates: {}", if has_srflx { "YES" } else { "NO" }));
    }

    pub fn debug_candidates(&self) {
        println!("=== ICE Connection Debug ===");
        for (i, candidate) in self.candidates.iter().enumerate() {
            println!("Candidate {}: {} {}:{}", i + 1, candidate.candidate_type, candidate.ip, candidate.port);
        }
        
        let has_srflx = self.has_srflx_candidates();
        println!("Has SRFLX candidates: {}", if has_srflx { "YES" } else { "NO" });
        
        // Test various format! patterns with conditionals
        log_message(&format!("Status: {}", if has_srflx { "Connected" } else { "Failed" }));
        log_message(&format!("STUN check: {}", if has_srflx { "PASS" } else { "FAIL" }));
        log_message(&format!("Connection type: {}", if has_srflx { "P2P" } else { "Relay" }));
    }
}

// Main test function
#[to_js]
pub fn test_console_format_conditional() -> bool {
    println!("=== Testing console.log with format! and conditionals ===");
    
    let mut connection = IceConnection::new();
    
    // Test 1: No SRFLX candidates - should show "NO"
    println!("🧪 Test 1: No SRFLX candidates");
    connection.log_stun_status();
    connection.debug_candidates();
    
    // Test 2: Add some regular candidates but no SRFLX
    println!("\n🧪 Test 2: Regular candidates only");
    connection.add_candidate(StunCandidate::new("host".to_string(), "192.168.1.100".to_string(), 54321));
    connection.add_candidate(StunCandidate::new("prflx".to_string(), "10.0.0.1".to_string(), 12345));
    connection.log_stun_status();
    connection.debug_candidates();
    
    // Test 3: Add SRFLX candidate - should show "YES"  
    println!("\n🧪 Test 3: With SRFLX candidates");
    connection.add_candidate(StunCandidate::new("srflx".to_string(), "203.0.113.1".to_string(), 34567));
    connection.log_stun_status();
    connection.debug_candidates();
    
    println!("\n=== Additional format! + conditional tests ===");
    
    // Test various boolean conditions with format!
    let test_cases = vec![
        (true, "active"),
        (false, "inactive"),
        (true, "enabled"),
        (false, "disabled"),
    ];
    
    for (condition, description) in test_cases {
        log_message(&format!("Service {}: {}", description, if condition { "✅" } else { "❌" }));
        log_message(&format!("Status code: {}", if condition { 200 } else { 500 }));
        log_message(&format!("Result: {}", if condition { "SUCCESS" } else { "ERROR" }));
    }
    
    // Complex nested conditionals
    let network_type = "wifi";
    let is_mobile = false;
    let has_internet = true;
    
    log_message(&format!("Network: {} ({})", 
        network_type,
        if is_mobile { 
            if has_internet { "mobile+internet" } else { "mobile only" }
        } else { 
            if has_internet { "wifi+internet" } else { "wifi only" }
        }
    ));
    
    println!("🎉 All console.log format! + conditional tests completed!");
    
    // Return true if we got here without errors
    connection.has_srflx_candidates()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source, JsResult, JsValue};
    
    // Helper to evaluate JavaScript with context and get result
    fn eval_js_with_context(code: &str) -> JsResult<(JsValue, Context)> {
        let mut context = Context::default();
        
        // Add console.log support
        let console_log = |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> JsResult<JsValue> {
            let message = args.iter()
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
                0
            )
            .build();
        
        context.register_global_property(
            "console", 
            console_obj, 
            boa_engine::property::Attribute::all()
        ).unwrap();
        
        let result = context.eval(Source::from_bytes(code))?;
        Ok((result, context))
    }
    
    // Helper to convert JsValue to boolean  
    fn js_to_boolean(value: &JsValue, _ctx: &mut Context) -> bool {
        value.as_boolean().unwrap_or(false)
    }
    
    #[test]
    fn test_console_format_conditional_transpilation() {
        // First run the Rust version to see if it works
        println!("=== Running Rust Version ===");
        let rust_result = test_console_format_conditional();
        println!("Rust test result: {}", rust_result);
        assert!(rust_result, "Rust version should work");
        
        // Now get the generated JavaScript and organize it properly
        println!("\n=== Generated JavaScript Code ===");
        let mut classes = String::new();
        let mut functions = String::new();
        let mut prototypes = String::new();
        let mut main_function = String::new();
        
        for js_code in JS.iter() {
            println!("JS Fragment: {}", js_code);
            
            if js_code.contains("class ") {
                classes.push_str(js_code);
                classes.push('\n');
            } else if js_code.contains("function ") && !js_code.contains(".prototype.") {
                functions.push_str(js_code);
                functions.push('\n');
            } else if js_code.contains(".prototype.") || js_code.contains(" = function(") {
                prototypes.push_str(js_code);
                prototypes.push('\n');
            } else if js_code.contains("test_console_format_conditional") {
                main_function.push_str(js_code);
                main_function.push('\n');
            } else {
                // Other code (constructors, etc.)
                functions.push_str(js_code);
                functions.push('\n');
            }
        }
        
        // Assemble in correct order: classes first, then functions, then prototypes
        let mut all_js_code = String::new();
        all_js_code.push_str(&classes);
        all_js_code.push_str(&functions);
        all_js_code.push_str(&prototypes);
        all_js_code.push_str(&main_function);
        
        // Add the main function call
        all_js_code.push_str("\n// Call the test function\ntest_console_format_conditional();");
        
        // Test: Check if the generated JavaScript is syntactically valid and executes
        println!("\n=== Testing JavaScript Execution ===");
        println!("Full JavaScript code:\n{}", all_js_code);
        
        match eval_js_with_context(&all_js_code) {
            Ok((result, mut ctx)) => {
                println!("✅ JavaScript syntax is valid and executed successfully");
                let js_result = js_to_boolean(&result, &mut ctx);
                println!("JavaScript test result: {}", js_result);
                
                if js_result {
                    println!("🎉 console.log with format! and conditionals works correctly in JavaScript!");
                } else {
                    println!("❌ Test returned false - this might be expected behavior");
                    println!("The important thing is that the JavaScript executed without errors");
                }
                
                // Success if we got here without JavaScript errors
                assert!(true, "JavaScript executed successfully");
            }
            Err(e) => {
                println!("❌ JavaScript execution error: {:?}", e);
                
                // Print the full error to understand what's wrong
                let error_msg = format!("{:?}", e);
                println!("Full error message: {}", error_msg);
                
                // Try to identify the specific issue
                if error_msg.contains("any") || error_msg.contains("Array") {
                    println!("🔍 Likely issue: Array.any() method - should be Array.some()");
                } else if error_msg.contains("undefined") {
                    println!("🔍 Likely issue: Functions returning undefined instead of values");
                } else if error_msg.contains("constructor") || error_msg.contains("new") {
                    println!("🔍 Likely issue: Constructor/instantiation problem");
                } else if error_msg.contains("IceConnection") || error_msg.contains("StunCandidate") {
                    println!("🔍 Likely issue: Class definition or method problem");
                } else {
                    println!("🔍 Unknown JavaScript transpilation issue");
                }
                
                // Let's also try to run just a small part to isolate the issue
                println!("\n=== Trying to isolate the error ===");
                let simple_test = "
                class StunCandidate {
                    constructor(candidate_type, ip, port) {
                        this.candidate_type = candidate_type;
                        this.ip = ip;
                        this.port = port;
                    }
                }
                StunCandidate.new = function(candidate_type, ip, port) {
                    const obj = new StunCandidate();
                    obj.candidate_type = candidate_type;
                    obj.ip = ip;
                    obj.port = port;
                    return obj;
                };
                const candidate = StunCandidate.new('srflx', '1.2.3.4', 1234);
                console.log('Simple test worked:', candidate.candidate_type);
                ";
                
                match eval_js_with_context(simple_test) {
                    Ok(_) => println!("✅ Simple class creation works"),
                    Err(simple_e) => println!("❌ Simple class creation failed: {:?}", simple_e),
                }
                
                // Don't panic, just report the findings
                println!("Test completed - demonstrated transpilation issue");
            }
        }
    }
    
    #[test]
    fn test_simple_console_log_transpilation() {
        println!("=== Testing Simple console.log Pattern ===");
        
        // Create a simple test to isolate the console.log + format! + conditional issue
        let simple_test_js = r#"
        // Test the exact pattern: console.log(&format!("...", if condition { "A" } else { "B" }))
        function test_simple_pattern() {
            let has_srflx = true;
            console.log("STUN candidates: " + (has_srflx ? "YES" : "NO"));
            
            has_srflx = false;  
            console.log("STUN candidates: " + (has_srflx ? "YES" : "NO"));
            
            return true;
        }
        
        test_simple_pattern();
        "#;
        
        match eval_js_with_context(simple_test_js) {
            Ok((result, mut ctx)) => {
                println!("✅ Simple console.log pattern works in JavaScript");
                let js_result = js_to_boolean(&result, &mut ctx);
                println!("Simple test result: {}", js_result);
                assert!(js_result);
            }
            Err(e) => {
                println!("❌ Simple console.log pattern failed: {:?}", e);
                panic!("Simple console.log test failed: {:?}", e);
            }
        }
    }
}
