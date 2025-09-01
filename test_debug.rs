use mojes_derive::{js_object, js_type};

#[js_type]
struct TestStruct;

#[js_object]
impl TestStruct {
    fn test_function(&self, participant_id: &str) {
        if true {
            let video_element = format!("video1-{}", participant_id);
            println!("Created video: {}", video_element);
        }
        if true {
            let video_element = format!("video2-{}", participant_id); 
            println!("Created video: {}", video_element); // Should use video_element_1?
        }
    }
}