use mojes_derive::{js_object, js_type};

#[js_type]
struct TestStruct {
    value: i32,
}

#[js_object]
impl TestStruct {
    // This should trigger the variable name conflict we're fixing
    fn setup_peer_connection_handlers(&self, pc: &mut i32, participant_id: &str) {
        let participant_id_clone = participant_id.to_string();
        let another_var = 42;
        
        // Use the variables to make sure they're properly tracked
        println!("Using participant_id: {}", participant_id);
        println!("Using participant_id_clone: {}", participant_id_clone);
        println!("Using another_var: {}", another_var);
    }
    
    // Test another pattern that could cause conflicts
    fn test_method(&self, data_channel: &str, participant_id: &str, connection_type: &str) {
        let data_channel = data_channel.to_uppercase(); // Should get renamed to data_channel_1
        let participant_id = participant_id.to_string(); // Should get renamed to participant_id_1
        
        println!("data_channel: {}", data_channel);
        println!("participant_id: {}", participant_id);
        println!("connection_type: {}", connection_type);
    }
}