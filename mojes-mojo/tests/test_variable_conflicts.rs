// Test for variable name conflict resolution
use mojes_mojo::*;
use syn::{ItemImpl, parse_quote};

#[test]
fn test_parameter_local_variable_conflict() {
    // Test the specific pattern that was causing issues:
    // fn method(&self, participant_id: &str) {
    //     let participant_id_clone = participant_id.to_string();
    // }
    
    let input_impl: ItemImpl = parse_quote! {
        impl TestStruct {
            fn setup_peer_connection_handlers(&self, participant_id: &str) {
                let participant_id_clone = participant_id.to_string();
                let another_var = 42;
            }
        }
    };

    // Generate JavaScript and check that no duplicate const declarations exist
    let js_code = generate_js_methods_for_impl(&input_impl);
    println!("Generated JavaScript:\n{}", js_code);
    
    // The JavaScript should NOT contain duplicate const declarations like:
    // const participant_id = participant_id.toString();
    // Instead it should have unique variable names
    assert!(!js_code.contains("const participant_id = participant_id.toString()"), 
           "Should not have duplicate variable declarations");
    
    // Should contain the renamed variable instead (participant_id_1 or similar)
    assert!(js_code.contains("participant_id_clone"), 
           "Should contain the participant_id_clone variable");
}

#[test]  
fn test_multiple_parameter_conflicts() {
    // Test multiple parameter conflicts in the same function
    let input_impl: ItemImpl = parse_quote! {
        impl TestStruct {
            fn test_method(&self, data_channel: &str, participant_id: &str, connection_type: &str) {
                let data_channel = data_channel.to_uppercase(); 
                let participant_id = participant_id.to_string();
                let connection_type = connection_type.trim();
            }
        }
    };

    let js_code = generate_js_methods_for_impl(&input_impl);
    println!("Generated JavaScript for multiple conflicts:\n{}", js_code);
    
    // Should not have any duplicate const declarations
    let lines: Vec<&str> = js_code.lines().collect();
    let const_lines: Vec<&str> = lines.iter().filter(|line| line.trim().starts_with("const ")).cloned().collect();
    
    // Extract variable names from const declarations
    let mut var_names: Vec<String> = Vec::new();
    for line in const_lines {
        if let Some(var_part) = line.trim().strip_prefix("const ") {
            if let Some(name) = var_part.split('=').next() {
                var_names.push(name.trim().to_string());
            }
        }
    }
    
    // Check for duplicates
    let mut unique_vars = std::collections::HashSet::new();
    for var_name in &var_names {
        assert!(unique_vars.insert(var_name.clone()), 
               "Duplicate variable declaration found: {}", var_name);
    }
    
    println!("Variable names found: {:?}", var_names);
}

#[test]
fn test_webrtc_specific_pattern() {
    // Test the specific pattern mentioned by user that was failing
    let input_impl: ItemImpl = parse_quote! {
        impl WebRTCManager {
            fn setup_data_channel_handlers(&self, data_channel: &str, participant_id: &str, connection_type: &str) {
                let channel_key = format!("{}-{}", participant_id, connection_type);
                let participant_id = participant_id.to_string();
                let connection_type = connection_type.to_string();
            }
        }
    };

    let js_code = generate_js_methods_for_impl(&input_impl);
    println!("Generated JavaScript for multiple conflicts:\n{}", js_code);
    
    // Should not have any duplicate const declarations
    let lines: Vec<&str> = js_code.lines().collect();
    let const_lines: Vec<&str> = lines.iter().filter(|line| line.trim().starts_with("const ")).cloned().collect();
    
    // Extract variable names from const declarations
    let mut var_names: Vec<String> = Vec::new();
    for line in const_lines {
        if let Some(var_part) = line.trim().strip_prefix("const ") {
            if let Some(name) = var_part.split('=').next() {
                var_names.push(name.trim().to_string());
            }
        }
    }
    
    // Check for duplicates
    let mut unique_vars = std::collections::HashSet::new();
    for var_name in &var_names {
        assert!(unique_vars.insert(var_name.clone()), 
               "Duplicate variable declaration found: {}", var_name);
    }
    
    println!("Variable names found: {:?}", var_names);
}