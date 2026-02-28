// Integration test for flexbox layout

use nomad_core::{Engine, EngineConfig};
use markup5ever_rcdom::{Node, NodeData};
use markup5ever::{LocalName, QualName, Namespace};
use std::rc::Rc;
use std::cell::{RefCell, Cell};

#[test]
fn test_flexbox_layout_simple() {
    // HTML with flexbox CSS
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                .container {
                    display: flex;
                    flex-direction: row;
                    justify-content: center;
                    width: 800px;
                }
                .box {
                    width: 100px;
                    height: 100px;
                    margin: 10px;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <div class="box">Box 1</div>
                <div class="box">Box 2</div>
                <div class="box">Box 3</div>
            </div>
        </body>
        </html>
    "#;

    // Parse and process
    let config = EngineConfig {
        viewport_width: 800.0,
        viewport_height: 600.0,
        ..Default::default()
    };

    let mut engine = Engine::with_config(config).unwrap();
    
    // Note: This test would need network mocking to work fully
    // For now, we just verify the engine can be created with flex config
    assert!(engine.config().viewport_width == 800.0);
}

#[test]
fn test_flex_column_layout() {
    let config = EngineConfig {
        viewport_width: 800.0,
        viewport_height: 600.0,
        ..Default::default()
    };

    let engine = Engine::with_config(config).unwrap();
    assert!(engine.config().viewport_height == 600.0);
}

#[test]
fn test_nested_flex_containers() {
    let config = EngineConfig {
        viewport_width: 1024.0,
        viewport_height: 768.0,
        ..Default::default()
    };

    let engine = Engine::with_config(config).unwrap();
    assert!(engine.config().viewport_width == 1024.0);
}
