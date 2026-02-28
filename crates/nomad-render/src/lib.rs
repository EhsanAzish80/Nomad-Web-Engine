//! Rendering engine for the Nomad Web Engine.
//!
//! Converts layout boxes into a display list for cross-platform rendering.

use nomad_layout::{LayoutBox, LayoutContent};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during rendering.
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// A display list containing all drawable items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayList {
    /// Width of the viewport
    pub width: f32,
    /// Total height of content
    pub height: f32,
    /// List of items to draw
    pub items: Vec<DisplayItem>,
}

impl DisplayList {
    /// Creates a new empty display list.
    pub fn new(width: f32) -> Self {
        Self {
            width,
            height: 0.0,
            items: Vec::new(),
        }
    }

    /// Serializes the display list to bytes (JSON format for cross-language compatibility).
    pub fn to_bytes(&self) -> Result<Vec<u8>, RenderError> {
        serde_json::to_vec(self)
            .map_err(|e| RenderError::Serialization(format!("Failed to serialize: {}", e)))
    }

    /// Deserializes a display list from bytes (JSON format).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RenderError> {
        serde_json::from_slice(bytes)
            .map_err(|e| RenderError::Serialization(format!("Failed to deserialize: {}", e)))
    }
}

/// A single drawable item in the display list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayItem {
    /// Type of item
    pub kind: DisplayItemKind,
    /// Bounding box
    pub bounds: Rect,
}

/// Type of display item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayItemKind {
    /// Text to draw
    Text {
        /// The text content
        content: String,
        /// Font size
        font_size: f32,
        /// Whether this is a link
        is_link: bool,
        /// URL if this is a link
        link_url: Option<String>,
    },
    /// Container box (for debugging/future use)
    Box {
        /// Background color (future)
        color: Option<String>,
    },
}

/// A rectangle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Checks if a point is inside this rectangle.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Render engine that converts layout boxes to display list.
pub struct RenderEngine {
    viewport_width: f32,
}

impl RenderEngine {
    /// Creates a new render engine.
    pub fn new(viewport_width: f32) -> Self {
        Self { viewport_width }
    }

    /// Convert a layout box tree into a display list.
    pub fn render(&self, layout_root: &LayoutBox) -> Result<DisplayList, RenderError> {
        let mut display_list = DisplayList::new(self.viewport_width);

        // Recursively add display items
        self.add_layout_box(&mut display_list, layout_root, 0.0, 0.0);

        // Update total height
        if let Some(max_item) = display_list.items.iter().max_by(|a, b| {
            let a_bottom = a.bounds.y + a.bounds.height;
            let b_bottom = b.bounds.y + b.bounds.height;
            a_bottom.partial_cmp(&b_bottom).unwrap()
        }) {
            display_list.height = max_item.bounds.y + max_item.bounds.height;
        }

        Ok(display_list)
    }

    /// Recursively add display items from layout boxes.
    fn add_layout_box(
        &self,
        display_list: &mut DisplayList,
        layout_box: &LayoutBox,
        offset_x: f32,
        offset_y: f32,
    ) {
        let abs_x = offset_x + layout_box.x;
        let abs_y = offset_y + layout_box.y;

        // Add display item based on content type
        match &layout_box.content {
            LayoutContent::Text {
                content,
                font_size,
                is_link,
                link_url,
            } => {
                if !content.trim().is_empty() {
                    display_list.items.push(DisplayItem {
                        kind: DisplayItemKind::Text {
                            content: content.clone(),
                            font_size: *font_size,
                            is_link: *is_link,
                            link_url: link_url.clone(),
                        },
                        bounds: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                    });
                }
            }
            LayoutContent::Element { .. } | LayoutContent::Anonymous => {
                // For elements, we might add a box later for backgrounds/borders
                // For now, just process children
            }
        }

        // Process children
        for child in &layout_box.children {
            self.add_layout_box(display_list, child, abs_x, abs_y);
        }
    }
}

impl Default for RenderEngine {
    fn default() -> Self {
        Self::new(800.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_engine_creation() {
        let engine = RenderEngine::new(800.0);
        assert_eq!(engine.viewport_width, 800.0);
    }

    #[test]
    fn test_display_list_serialization() {
        let mut list = DisplayList::new(800.0);
        list.items.push(DisplayItem {
            kind: DisplayItemKind::Text {
                content: "Hello".to_string(),
                font_size: 16.0,
                is_link: false,
                link_url: None,
            },
            bounds: Rect::new(20.0, 20.0, 50.0, 16.0),
        });

        // Serialize and deserialize
        let bytes = list.to_bytes().unwrap();
        let deserialized = DisplayList::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.width, 800.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(50.0, 30.0));
        assert!(!rect.contains(5.0, 30.0));
        assert!(!rect.contains(50.0, 5.0));
    }
}
