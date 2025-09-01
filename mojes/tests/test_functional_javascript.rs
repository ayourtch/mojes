use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Mock functions to simulate real behavior
fn create_participant_video_element(participant_id: &str) -> Option<String> {
    Some(format!("video_{}", participant_id))
}

fn debug_repr(value: &Option<String>) -> String {
    match value {
        Some(v) => format!("Some({})", v),
        None => "None".to_string(),
    }
}

#[js_type]
#[derive(Debug, Clone)]
pub struct FunctionalTestManager {
    pub test_results: Vec<String>,
}

#[js_object]
impl FunctionalTestManager {
    pub fn new() -> Self {
        Self {
            test_results: vec![],
        }
    }

    // Test tuple destructuring in if let patterns
    pub fn test_tuple_destructuring(&mut self, participant_id: &str) -> bool {
        let get_video_and_audio = |id: &str| -> Option<(String, String)> {
            Some((format!("video_{}", id), format!("audio_{}", id)))
        };

        let mut success = true;
        
        // Test 1: First tuple destructuring
        if let Some((video_element, audio_element)) = get_video_and_audio(participant_id) {
            let expected_video = format!("video_{}", participant_id);
            let expected_audio = format!("audio_{}", participant_id);
            
            if video_element == expected_video && audio_element == expected_audio {
                println!("✅ Tuple destructuring test 1 passed: video={}, audio={}", video_element, audio_element);
            } else {
                println!("❌ Tuple destructuring test 1 failed: expected video={}, audio={}, got video={}, audio={}", 
                         expected_video, expected_audio, video_element, audio_element);
                success = false;
            }
        } else {
            println!("❌ Tuple destructuring test 1 failed: None returned");
            success = false;
        }

        // Test 2: Second tuple destructuring with different input
        let second_id = format!("second_{}", participant_id);
        if let Some((video_element, audio_element)) = get_video_and_audio(&second_id) {
            let expected_video = format!("video_{}", second_id);
            let expected_audio = format!("audio_{}", second_id);
            
            if video_element == expected_video && audio_element == expected_audio {
                println!("✅ Tuple destructuring test 2 passed: video={}, audio={}", video_element, audio_element);
            } else {
                println!("❌ Tuple destructuring test 2 failed: expected video={}, audio={}, got video={}, audio={}", 
                         expected_video, expected_audio, video_element, audio_element);
                success = false;
            }
        } else {
            println!("❌ Tuple destructuring test 2 failed: None returned");
            success = false;
        }

        success
    }

    // Test template literal variable references
    pub fn test_template_literals(&self, participant_id: &str) -> bool {
        let mut success = true;
        
        // Test 1: First if block
        if true {
            let video_element = create_participant_video_element(participant_id);
            let result = format!("Template 1: Video is: {}", debug_repr(&video_element));
            let expected = format!("Template 1: Video is: Some(video_{})", participant_id);
            
            if result == expected {
                println!("✅ Template literal test 1 passed: {}", result);
            } else {
                println!("❌ Template literal test 1 failed: expected '{}', got '{}'", expected, result);
                success = false;
            }
        }

        // Test 2: Second if block (should not conflict)
        if true {
            let video_element = create_participant_video_element(&format!("alt_{}", participant_id));
            let result = format!("Template 2: Video is: {}", debug_repr(&video_element));
            let expected = format!("Template 2: Video is: Some(video_alt_{})", participant_id);
            
            if result == expected {
                println!("✅ Template literal test 2 passed: {}", result);
            } else {
                println!("❌ Template literal test 2 failed: expected '{}', got '{}'", expected, result);
                success = false;
            }
        }

        success
    }

    // Test variable scoping and renaming
    pub fn test_variable_scoping(&self, participant_id: &str) -> bool {
        let mut success = true;
        
        // Test same scope conflict resolution
        let video_element = create_participant_video_element(participant_id);
        let first_result = debug_repr(&video_element);
        
        let video_element = format!("different_{}", participant_id); // Should be renamed to video_element_1
        let second_result = video_element;
        
        let expected_first = format!("Some(video_{})", participant_id);
        let expected_second = format!("different_{}", participant_id);
        
        if first_result == expected_first && second_result == expected_second {
            println!("✅ Variable scoping test passed: first='{}', second='{}'", first_result, second_result);
        } else {
            println!("❌ Variable scoping test failed: expected first='{}', second='{}', got first='{}', second='{}'", 
                     expected_first, expected_second, first_result, second_result);
            success = false;
        }

        success
    }

    // Test the is_some() IIFE optimization
    pub fn test_is_some_optimization(&self, participant_id: &str) -> bool {
        let mut call_count = 0;
        
        // Simulate a function that we want to call only once
        let mut expensive_function = || {
            call_count += 1;
            println!("📞 expensive_function called {} time(s)", call_count);
            Some(format!("expensive_result_{}", participant_id))
        };

        // This should only call expensive_function once due to IIFE optimization
        let result = expensive_function().is_some();
        
        if call_count == 1 && result {
            println!("✅ is_some IIFE optimization test passed: function called {} time, result={}", call_count, result);
            true
        } else {
            println!("❌ is_some IIFE optimization test failed: function called {} times, result={}", call_count, result);
            false
        }
    }
}

// Main test runner
#[to_js]
pub fn test_functional_javascript() -> bool {
    println!("=== Running Functional JavaScript Tests ===");
    
    let mut manager = FunctionalTestManager::new();
    let mut all_passed = true;
    
    println!("\n🧪 Test 1: Tuple Destructuring");
    if !manager.test_tuple_destructuring("user123") {
        all_passed = false;
    }
    
    println!("\n🧪 Test 2: Template Literals");
    if !manager.test_template_literals("user456") {
        all_passed = false;
    }
    
    println!("\n🧪 Test 3: Variable Scoping");
    if !manager.test_variable_scoping("user789") {
        all_passed = false;
    }
    
    println!("\n🧪 Test 4: is_some IIFE Optimization");
    if !manager.test_is_some_optimization("user000") {
        all_passed = false;
    }
    
    println!("\n=== Test Results ===");
    if all_passed {
        println!("🎉 All functional JavaScript tests PASSED!");
    } else {
        println!("💥 Some functional JavaScript tests FAILED!");
    }
    
    all_passed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::fs;
    
    #[test] 
    fn test_functional_javascript_execution() {
        // First run the transpiler to generate JavaScript
        test_functional_javascript();
        
        // Extract the generated JavaScript
        println!("\n=== Generated JavaScript Code ===");
        let mut all_js = String::new();
        for js_code in JS.iter() {
            all_js.push_str(js_code);
            all_js.push('\n');
        }
        
        // Split the JavaScript to reorder it properly
        let mut function_code = String::new();
        let mut class_code = String::new();
        let mut method_code = String::new();
        
        for js_code in JS.iter() {
            if js_code.contains("function test_functional_javascript") {
                function_code.push_str(js_code);
                function_code.push('\n');
            } else if js_code.contains("class FunctionalTestManager") {
                class_code.push_str(js_code);
                class_code.push('\n');
            } else if js_code.contains("FunctionalTestManager.") {
                method_code.push_str(js_code);
                method_code.push('\n');
            }
        }
        
        // Write JavaScript to a temporary file with proper ordering
        let js_content = format!("
// Mock the functions that exist in Rust
function create_participant_video_element(participant_id) {{
    return `video_${{participant_id}}`;
}}

function debug_repr(value) {{
    if (value === null || value === undefined) {{
        return 'None';
    }} else {{
        return `Some(${{value}})`;
    }}
}}

// Define classes first
{}

// Then define static methods
{}

// Then define functions
{}

// Execute the test
console.log('Starting JavaScript execution test...');
const result = test_functional_javascript();
console.log(`Final result: ${{result}}`);
process.exit(result ? 0 : 1);
", class_code, method_code, function_code);
        
        fs::write("test_functional.js", js_content).expect("Failed to write JS file");
        
        // Execute the JavaScript file with Node.js
        println!("\n=== Executing JavaScript with Node.js ===");
        let output = Command::new("node")
            .arg("test_functional.js")
            .output()
            .expect("Failed to execute node command");
            
        println!("STDOUT:\n{}", String::from_utf8_lossy(&output.stdout));
        println!("STDERR:\n{}", String::from_utf8_lossy(&output.stderr));
        
        // Clean up
        let _ = fs::remove_file("test_functional.js");
        
        // Check if the JavaScript execution was successful
        assert!(output.status.success(), "JavaScript execution failed!");
        
        // Also check that the output contains success indicators
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("All functional JavaScript tests PASSED!"), 
                "JavaScript tests did not all pass: {}", stdout);
    }
}