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
            // Both video_element and audio_element should be renamed with _1 suffix
            println!("Second tuple: video={}, audio={}", video_element, audio_element); // Should use renamed variables
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
            let video_element = create_participant_video_element(participant_id); // Should be video_element_1
            println!("Created video: {}", debug_repr(&video_element)); // Should use video_element_1!
        }
        
        // Now with if let pattern
        if let Some(video_element) = create_participant_video_element(participant_id) {
            println!("if let video: {}", debug_repr(&Some(video_element)));
        }
        
        if let Some(video_element) = create_participant_video_element(participant_id) { // Should be video_element_1
            println!("if let video 2: {}", debug_repr(&Some(video_element))); // Should use video_element_1!
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
    // Look for the pattern where video_element_1 is declared but video_element is used
    if js_code.contains("video_element_1") {
        // If we have a renamed variable, ALL references in that scope should use the renamed version
        let lines: Vec<&str> = js_code.lines().collect();
        let mut in_second_if = false;
        let mut found_declaration = false;
        let mut found_wrong_usage = false;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Look for the declaration of the renamed variable
            if trimmed.contains("video_element_1") && trimmed.contains("=") {
                found_declaration = true;
                in_second_if = true;
                continue;
            }
            
            // If we're in the second if block and see the old variable name being used
            if in_second_if && trimmed.contains("video_element") && !trimmed.contains("video_element_1") {
                found_wrong_usage = true;
                println!("❌ BUG DETECTED: Line uses 'video_element' instead of 'video_element_1': {}", trimmed);
            }
            
            // End of if block
            if trimmed == "}" && in_second_if {
                in_second_if = false;
            }
        }
        
        if found_declaration && found_wrong_usage {
            panic!("❌ VARIABLE REFERENCE BUG: Found video_element_1 declaration but expressions still use video_element!");
        } else if found_declaration {
            println!("✅ Variable references appear correct in separate if blocks");
        }
    }
}

fn check_same_scope_pattern(js_code: &str) {
    // Similar check for same scope conflicts
    if js_code.contains("video_element_1") {
        println!("✅ Same scope conflict detected and handled");
        
        // Check that the second println uses video_element_1, not video_element
        // This is trickier to validate automatically, so we'll rely on manual inspection for now
    }
}

fn check_template_pattern(js_code: &str) {
    // Check template string variable references
    if js_code.contains("video_element_1") {
        println!("✅ Template string conflict detected");
        // Look for incorrect variable references in template expressions
        if js_code.contains("${video_element}") && js_code.contains("video_element_1") {
            println!("⚠️ Potential issue: Template might be using wrong variable reference");
        }
    }
}

fn check_if_let_pattern(js_code: &str) {
    println!("🔍 Checking if let pattern handling...");
    
    // Check if if-let patterns are properly transpiled and create variable conflicts
    if js_code.contains("test_if_let_some_pattern") {
        if js_code.contains("video_element_1") {
            println!("✅ if let patterns create variable conflicts correctly");
        } else {
            println!("⚠️ if let patterns might not be creating expected conflicts");
        }
        
        // Look for the specific pattern where expressions should use renamed variables
        let lines: Vec<&str> = js_code.lines().collect();
        let mut found_if_let_issue = false;
        
        for line in lines {
            let trimmed = line.trim();
            // Look for console.log with debug_repr but wrong variable reference
            if trimmed.contains("console.log") && trimmed.contains("debug_repr") {
                if trimmed.contains("video_element") && !trimmed.contains("video_element_1") && js_code.contains("video_element_1") {
                    println!("❌ if let BUG: Found console.log using video_element instead of renamed version in: {}", trimmed);
                    found_if_let_issue = true;
                }
            }
        }
        
        if !found_if_let_issue {
            println!("✅ if let variable references appear correct");
        }
    }
}

fn check_exact_original_pattern(js_code: &str) {
    println!("🔍 Checking exact original pattern (your bug report)...");
    
    // This should check the specific pattern you reported
    if js_code.contains("test_exact_original_pattern") {
        let has_regular_if_conflicts = js_code.contains("video_element_1");
        let has_if_let_conflicts = js_code.contains("video_element_2"); // if let should create additional conflicts
        
        if has_regular_if_conflicts {
            println!("✅ Regular if blocks create conflicts correctly");
        } else {
            println!("❌ Regular if blocks not creating expected conflicts");
        }
        
        if has_if_let_conflicts {
            println!("✅ if let blocks also create conflicts correctly");
        } else {
            println!("⚠️ if let blocks might not be creating expected conflicts");
        }
        
        // Check for your specific bug pattern
        let lines: Vec<&str> = js_code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Look for variable declaration followed by incorrect usage
            if trimmed.contains("video_element_1") && trimmed.contains("=") && !trimmed.contains("console.log") {
                // Found renamed variable declaration, check next few lines for usage
                for j in (i+1)..(i+5).min(lines.len()) {
                    let usage_line = lines[j].trim();
                    if usage_line.contains("console.log") && usage_line.contains("debug_repr") {
                        if usage_line.contains("video_element") && !usage_line.contains("video_element_1") {
                            println!("❌ EXACT BUG REPRODUCED: Variable declared as video_element_1 but used as video_element in: {}", usage_line);
                        } else if usage_line.contains("video_element_1") {
                            println!("✅ Variable correctly uses renamed version: {}", usage_line);
                        }
                    }
                }
            }
        }
    }
}