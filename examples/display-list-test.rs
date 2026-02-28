//! Test to output display list JSON for debugging

use nomad_core::Engine;

fn main() {
    let mut engine = Engine::new().expect("Engine creation failed");
    
    // Set viewport width
    engine.set_viewport_width(800.0);
    
    // Load Google
    println!("Loading https://google.com...\n");
    engine.load_url("https://google.com").expect("Failed to load URL");
    
    // Get display list as JSON
    let json = engine.get_display_list_json().expect("Failed to get display list");
    
    println!("Display List JSON:");
    println!("{}", json);
}
