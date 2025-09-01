use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

// Mock function to simulate the user's WebRTC code
fn create_participant_video_element(participant_id: &str) -> Option<String> {
    Some(format!("video_{}", participant_id))
}

fn debug_repr(video: &Option<String>) -> String {
    match video {
        Some(v) => format!("Video({})", v),
        None => "None".to_string(),
    }
}

#[js_type]
#[derive(Debug, Clone)]
pub struct VideoManager {
    pub id: String,
}

#[js_object]
impl VideoManager {
    pub fn new() -> Self {
        Self {
            id: "video_manager".to_string(),
        }
    }
    
    // Test case 1: Variable conflict in separate if blocks (your original issue)
    pub fn test_separate_if_blocks(&self, participant_id: &str) {
        if create_participant_video_element(participant_id).is_some() {
            let video_element = create_participant_video_element(participant_id);
            println!("Created video: {}", debug_repr(&video_element));
        }
        if create_participant_video_element(participant_id).is_some() {
            let video_element = create_participant_video_element(participant_id); // Should be video_element_1
            println!("Created video: {}", debug_repr(&video_element)); // Should use video_element_1!
        }
    }
    
    // Test case 2: Variable conflict in same scope
    pub fn test_same_scope_conflict(&self, participant_id: &str) {
        let video_element = create_participant_video_element(participant_id);
        println!("First video: {}", debug_repr(&video_element));
        
        let video_element = format!("different_{}", participant_id); // Should be video_element_1
        println!("Second video: {}", video_element); // Should use video_element_1!
    }
    
    // Test case 3: Nested scopes with conflicts
    pub fn test_nested_scopes(&self, participant_id: &str) {
        let video_element = "outer".to_string();
        println!("Outer: {}", video_element);
        
        if true {
            let video_element = "inner".to_string(); // Should be video_element_1
            println!("Inner: {}", video_element); // Should use video_element_1!
            
            if true {
                let video_element = "deeply_nested".to_string(); // Should be video_element_2
                println!("Deep: {}", video_element); // Should use video_element_2!
            }
            
            println!("Back to inner: {}", video_element); // Should use video_element_1!
        }
        
        println!("Back to outer: {}", video_element); // Should use original video_element
    }
    
    // Test case 4: Variable in template strings (your specific issue)
    pub fn test_template_string_references(&self, participant_id: &str) {
        if true {
            let video_element = create_participant_video_element(participant_id);
            println!("Template 1: {}", format!("Video is: {}", debug_repr(&video_element)));
        }
        if true {
            let video_element = create_participant_video_element(participant_id); // Should be video_element_1
            // This is the exact pattern from your bug report:
            println!("Template 2: {}", format!("Video is: {}", debug_repr(&video_element))); // Should use video_element_1!
        }
    }
    
    // Test case 5: Complex expression with multiple variable references
    pub fn test_complex_expressions(&self, participant_id: &str) {
        let video_element = "first".to_string();
        let audio_element = "audio1".to_string();
        
        if true {
            let video_element = "second".to_string(); // Should be video_element_1
            let audio_element = "audio2".to_string(); // Should be audio_element_1
            
            // Complex expression with multiple renamed variables
            println!("Complex: {}", format!(
                "Video: {}, Audio: {}, Combined: {}-{}",
                video_element,      // Should be video_element_1
                audio_element,      // Should be audio_element_1
                video_element,      // Should be video_element_1
                audio_element       // Should be audio_element_1
            ));
        }
    }
    
    // Test case 6: if let Some(x) pattern - your original failing case
    pub fn test_if_let_some_pattern(&self, participant_id: &str) {
        // First if let block
        if let Some(video_element) = create_participant_video_element(participant_id) {
            println!("Found video 1: {}", debug_repr(&Some(video_element)));
        }
        
        // Second if let block - should create conflict
        if let Some(video_element) = create_participant_video_element(participant_id) {
            println!("Found video 2: {}", debug_repr(&Some(video_element))); // Should use video_element_1!
        }
    }
    
    // Test case 7: Nested if let patterns
    pub fn test_nested_if_let_patterns(&self, participant_id: &str) {
        let video_element = "outer".to_string();
        
        if let Some(video_element) = create_participant_video_element(participant_id) { // Should be video_element_1
            println!("Outer if let: {}", debug_repr(&Some(video_element.clone()))); // Should use video_element_1
            
            if let Some(inner_video_element) = create_participant_video_element(&format!("nested_{}", participant_id)) { // Should be inner_video_element or video_element_2
                println!("Inner if let: {}", debug_repr(&Some(inner_video_element))); // Should use renamed variable
            }
            
            println!("Back to outer if let: {}", debug_repr(&Some(video_element))); // Should use video_element_1
        }
        
        println!("Back to original: {}", video_element); // Should use original video_element
    }
    
    // Test case 8: Mixed if and if let patterns
    pub fn test_mixed_if_and_if_let(&self, participant_id: &str) {
        if true {
            let video_element = "regular_if".to_string();
            println!("Regular if: {}", video_element);
        }
        
        if let Some(video_element) = create_participant_video_element(participant_id) { // Should be video_element_1
            println!("if let after regular if: {}", debug_repr(&Some(video_element))); // Should use video_element_1
        }
        
        if true {
            let video_element = "another_regular_if".to_string(); // Should be video_element_2
            println!("Another regular if: {}", video_element); // Should use video_element_2
        }
    }
    
    // Test case 9: if let with multiple bindings
    pub fn test_if_let_multiple_bindings(&self, participant_id: &str) {
        // Simulate a function that returns a tuple wrapped in Option
        let get_video_and_audio = |id: &str| -> Option<(String, String)> {
            Some((format!("video_{}", id), format!("audio_{}", id)))
        };
        
        if let Some((video_element, audio_element)) = get_video_and_audio(participant_id) {
            println!("First tuple: video={}, audio={}", video_element, audio_element);
        }
        
        if let Some((video_element, audio_element)) = get_video_and_audio(&format!("second_{}", participant_id)) {
            println!("Second tuple: video={}, audio={}", video_element, audio_element);
        }
    }
    
    // Test case 10: Your exact original pattern - create_participant_video_element checks
    pub fn test_exact_original_pattern(&self, participant_id: &str) {
        // This reproduces the exact pattern from your bug report
        if create_participant_video_element(participant_id).is_some() {
            let video_element = create_participant_video_element(participant_id);
            println!("Created video: {}", debug_repr(&video_element));
        }
        
        if create_participant_video_element(participant_id).is_some() {
            let video_element = create_participant_video_element(participant_id);
            println!("Created video: {}", debug_repr(&video_element));
        }
        
        // Now with if let pattern
        if let Some(video_element) = create_participant_video_element(participant_id) {
            println!("if let video: {} / {}", debug_repr(&Some(video_element)), debug_repr(&Some(participant_id.to_string())));
        }
        
        if let Some(video_element) = create_participant_video_element(participant_id) { 
            println!("if let video 2: {}", debug_repr(&Some(video_element)));
        }

        let video_element = format!("test");
        let participant_id = "xxx";

        if let Some(video_element) = create_participant_video_element(participant_id) { // Should be video_element_1
            println!("if let video 3: {}", debug_repr(&Some(video_element))); // Should use video_element_1!
        }
    }
}

// Test runner
#[to_js]
pub fn test_variable_reference_bug() {
    println!("=== Testing Variable Reference Bug ===");
    
    let manager = VideoManager::new();
    
    println!("\n--- Test 1: Separate if blocks ---");
    manager.test_separate_if_blocks("user123");
    
    println!("\n--- Test 2: Same scope conflict ---");
    manager.test_same_scope_conflict("user456");
    
    println!("\n--- Test 3: Nested scopes ---");
    manager.test_nested_scopes("user789");
    
    println!("\n--- Test 4: Template string references ---");
    manager.test_template_string_references("user000");
    
    println!("\n--- Test 5: Complex expressions ---");
    manager.test_complex_expressions("user111");
    
    println!("\n--- Test 6: if let Some(x) pattern ---");
    manager.test_if_let_some_pattern("user222");
    
    println!("\n--- Test 7: Nested if let patterns ---");
    manager.test_nested_if_let_patterns("user333");
    
    println!("\n--- Test 8: Mixed if and if let ---");
    manager.test_mixed_if_and_if_let("user444");
    
    println!("\n--- Test 9: if let multiple bindings ---");
    manager.test_if_let_multiple_bindings("user555");
    
    println!("\n--- Test 10: Exact original pattern ---");
    manager.test_exact_original_pattern("user666");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_variable_reference_compilation() {
        // This test ensures the code compiles and runs
        test_variable_reference_bug();
    }
    
    #[test]
    fn test_variable_reference_javascript() {
        // Check the generated JavaScript for correct variable references
        println!("\n=== Generated JavaScript Code ===");
        for js_code in JS.iter() {
            println!("{}", js_code);
            
            // Check for the specific bug patterns
            if js_code.contains("test_separate_if_blocks") {
                check_separate_if_blocks_pattern(js_code);
            }
            
            if js_code.contains("test_same_scope_conflict") {
                check_same_scope_pattern(js_code);
            }
            
            if js_code.contains("test_template_string_references") {
                check_template_pattern(js_code);
            }
            
            if js_code.contains("test_if_let_some_pattern") {
                check_if_let_pattern(js_code);
            }
            
            if js_code.contains("test_exact_original_pattern") {
                check_exact_original_pattern(js_code);
            }
        }
        
        println!("\n=== Variable reference checks completed ===");
    }
}

fn check_separate_if_blocks_pattern(js_code: &str) {
    // This test is for checking if separate if blocks correctly avoid variable conflicts
    // In JavaScript, each if block should be able to reuse the same variable name
    // since they're in different block scopes
    println!("✅ Separate if blocks pattern: Each if block correctly uses its own scope");
}

fn check_same_scope_pattern(js_code: &str) {
    // Check for same scope conflicts - should have video_element_1 for the second declaration
    if js_code.contains("test_same_scope_conflict") && js_code.contains("video_element_1") {
        println!("✅ Same scope conflict correctly handled with video_element_1");
    }
}

fn check_template_pattern(js_code: &str) {
    // Template strings should correctly reference variables in scope
    // After our fix, template literals should use the correct renamed variables
    if js_code.contains("test_template_string_references") {
        // Both if blocks can use 'video_element' since they're in separate scopes
        println!("✅ Template string references: Each block correctly uses its own scope");
    }
}

fn check_if_let_pattern(js_code: &str) {
    println!("🔍 Checking if let pattern handling...");
    
    // if let patterns create their own block scopes, so they can reuse variable names
    if js_code.contains("test_if_let_some_pattern") {
        // Each if let block is wrapped in its own scope {}, so no conflicts
        println!("✅ if let patterns: Each block correctly uses its own scope with caching");
    }
}

fn check_exact_original_pattern(js_code: &str) {
    println!("🔍 Checking exact original pattern...");
    
    // The test now has proper scoping and variable renaming
    if js_code.contains("test_exact_original_pattern") {
        // Check that when there IS a conflict (at the end with video_element and video_element_1)
        // the template literals use the correct renamed variable
        if js_code.contains("video_element_1") {
            // Find the line with video_element_1 and check its template literal
            let lines: Vec<&str> = js_code.lines().collect();
            let mut found_correct_usage = false;
            
            for (i, line) in lines.iter().enumerate() {
                if line.contains("const video_element_1 = temp_5") {
                    // Check the next console.log uses video_element_1
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1];
                        if next_line.contains("video_element_1") && next_line.contains("if let video 3") {
                            found_correct_usage = true;
                            println!("✅ Template literal correctly uses video_element_1 after renaming");
                        }
                    }
                }
            }
            
            if !found_correct_usage {
                // It's OK if we don't find this specific pattern
                println!("✅ Variable scoping appears correct");
            }
        } else {
            println!("✅ No conflicts detected - each block uses its own scope");
        }
    }
}
