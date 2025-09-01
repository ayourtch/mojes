use mojes_derive::{to_js, js_object, js_type};
use linkme::distributed_slice;

#[distributed_slice]
static JS: [&str] = [..];

#[js_type]
#[derive(Debug, Clone)]
pub struct WebRTCManager {
    pub connection_id: String,
}

#[js_object]
impl WebRTCManager {
    pub fn new() -> Self {
        Self {
            connection_id: "default".to_string(),
        }
    }
    
    // Test the exact pattern that was causing issues
    pub fn setup_data_channel_handlers(&self, data_channel: &str, participant_id: &str, connection_type: &str) {
        let channel_key = format!("{}-{}", participant_id, connection_type);
        let participant_id = participant_id.to_string();
        let connection_type = connection_type.to_string();
        
        println!("Channel key: {}", channel_key);
        println!("Participant ID: {}", participant_id);
        println!("Connection type: {}", connection_type);
    }
    
    // Test multiple parameter conflicts in same function  
    pub fn setup_peer_connection_handlers(&self, pc: &str, participant_id: &str) {
        let participant_id_clone = participant_id.to_string();
        let another_var = 42;
        let pc = pc.to_uppercase(); // This should get renamed to pc_1
        
        println!("PC: {}", pc);
        println!("Participant ID clone: {}", participant_id_clone);
        println!("Another var: {}", another_var);
    }
    
    // Test with template string usage (the specific failing case)
    pub fn create_channel_key(&self, participant_id: &str, connection_type: &str) {
        // This was generating: const channel_key = `${participant_id_1}-${connection_type_1}`;
        // But should generate: const channel_key = `${participant_id}-${connection_type}`;
        let channel_key = format!("{}-{}", participant_id, connection_type);
        
        // These should get unique names
        let participant_id = participant_id.to_string();
        let connection_type = connection_type.trim().to_string();
        
        println!("Channel key: {}", channel_key);
        println!("Processed participant_id: {}", participant_id);
        println!("Processed connection_type: {}", connection_type);
    }
}

// Test function to run the variable conflict scenarios
#[to_js]
pub fn test_variable_conflicts() {
    println!("=== Testing Variable Conflict Resolution ===");
    
    let manager = WebRTCManager::new();
    
    println!("\n--- Test 1: setup_data_channel_handlers ---");
    manager.setup_data_channel_handlers("channel1", "user123", "reliable");
    
    println!("\n--- Test 2: setup_peer_connection_handlers ---");
    manager.setup_peer_connection_handlers("pc1", "user456");
    
    println!("\n--- Test 3: create_channel_key ---");
    manager.create_channel_key("user789", "unreliable");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_variable_conflict_compilation() {
        // This test ensures the code compiles and runs without JavaScript conflicts
        test_variable_conflicts();
    }
    
    #[test]
    fn test_generated_javascript() {
        // Print all collected JavaScript to verify the transpilation
        println!("\n=== Generated JavaScript Code ===");
        for js_code in JS.iter() {
            println!("{}", js_code);
            
            // Check that we don't have duplicate const declarations within each method
            check_for_method_level_duplicates(js_code);
            
            // Specific checks for the problematic patterns
            if js_code.contains("setup_data_channel_handlers") {
                // Should NOT contain: const participant_id = participant_id_1.toString();
                // Should contain: const participant_id_1 = participant_id.toString();
                assert!(!js_code.contains("const participant_id = participant_id_1"), 
                       "Found backward variable assignment in setup_data_channel_handlers");
                assert!(!js_code.contains("const connection_type = connection_type_1"), 
                       "Found backward variable assignment in setup_data_channel_handlers");
            }
            
            if js_code.contains("create_channel_key") {
                // Template strings should use original parameter names (which they do)
                if js_code.contains("${") {
                    // Check that channel_key template uses original parameter names
                    if js_code.contains("const channel_key = `${") {
                        assert!(js_code.contains("const channel_key = `${participant_id}-${connection_type}`"), 
                               "Channel key template should use original parameter names");
                    }
                    // Note: Console.log templates correctly use renamed variables for processed values
                }
            }
        }
        
        println!("\n=== All variable conflict checks passed! ===");
    }
}

// Helper function to check for duplicates within each method, not across methods
fn check_for_method_level_duplicates(js_code: &str) {
    let lines: Vec<&str> = js_code.lines().collect();
    
    // Split into individual methods for separate duplicate checking
    let mut current_method_lines: Vec<&str> = Vec::new();
    let mut method_name = String::new();
    let mut in_method = false;
    let mut brace_count = 0;
    
    for line in lines {
        let trimmed = line.trim();
        
        // Start of a method
        if trimmed.contains(".prototype.") && trimmed.contains("= function") {
            // Process previous method if any
            if !current_method_lines.is_empty() {
                check_method_for_duplicates(&current_method_lines, &method_name, js_code);
                current_method_lines.clear();
            }
            
            // Extract method name
            if let Some(start) = trimmed.find(".prototype.") {
                if let Some(end) = trimmed[start + 11..].find(" =") {
                    method_name = trimmed[start + 11..start + 11 + end].to_string();
                }
            }
            
            in_method = true;
            brace_count = 0;
            current_method_lines.push(line);
        } else if in_method {
            current_method_lines.push(line);
            
            // Count braces to find method end
            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;
            
            // End of method when braces balance
            if brace_count <= 0 && trimmed.ends_with("};") {
                check_method_for_duplicates(&current_method_lines, &method_name, js_code);
                current_method_lines.clear();
                in_method = false;
            }
        }
    }
    
    // Process final method if any
    if !current_method_lines.is_empty() {
        check_method_for_duplicates(&current_method_lines, &method_name, js_code);
    }
}

fn check_method_for_duplicates(method_lines: &[&str], method_name: &str, full_js_code: &str) {
    let const_lines: Vec<&str> = method_lines.iter()
        .filter(|line| line.trim().starts_with("const "))
        .cloned()
        .collect();
    
    // Extract variable names from const declarations
    let mut var_names: Vec<String> = Vec::new();
    for line in const_lines {
        if let Some(var_part) = line.trim().strip_prefix("const ") {
            if let Some(name) = var_part.split('=').next() {
                let name = name.trim().to_string();
                if !name.is_empty() {
                    var_names.push(name);
                }
            }
        }
    }
    
    // Check for duplicates within this method only
    let mut unique_vars = std::collections::HashSet::new();
    for var_name in &var_names {
        if !unique_vars.insert(var_name.clone()) {
            panic!("Duplicate const declaration found within method '{}': '{}'\n\nMethod code:\n{}\n\nFull JS code:\n{}", 
                   method_name, var_name, method_lines.join("\n"), full_js_code);
        }
    }
}