use nomad_core::display_list::{DisplayItem, DisplayItemKind, DisplayList, Rect};

fn main() {
    // Create a sample display list
    let mut list = DisplayList::new(800.0);
    
    list.add_item(DisplayItem {
        kind: DisplayItemKind::Text {
            content: "Hello World".to_string(),
            font_size: 16.0,
            is_link: false,
            link_url: None,
        },
        bounds: Rect::new(20.0, 20.0, 100.0, 16.0),
    });
    
    list.add_item(DisplayItem {
        kind: DisplayItemKind::Text {
            content: "Click here".to_string(),
            font_size: 16.0,
            is_link: true,
            link_url: Some("https://example.com".to_string()),
        },
        bounds: Rect::new(20.0, 40.0, 80.0, 16.0),
    });
    
    // Serialize to JSON
    let json_bytes = list.to_bytes().expect("Failed to serialize");
    let json_string = String::from_utf8(json_bytes).expect("Invalid UTF-8");
    
    println!("Display list JSON:");
    println!("{}", json_string);
    
    // Pretty print
    println!("\nPretty printed:");
    let pretty = serde_json::to_string_pretty(&list).expect("Failed to pretty print");
    println!("{}", pretty);
}
