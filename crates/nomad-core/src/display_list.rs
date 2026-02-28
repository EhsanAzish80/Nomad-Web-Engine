//! Display list for rendering.
//!
//! Contains structures for representing visual output that can be
//! serialized across the FFI boundary.

use serde::{Deserialize, Serialize};

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

    /// Adds an item to the display list.
    pub fn add_item(&mut self, item: DisplayItem) {
        // Update height based on item bounds
        let item_bottom = item.bounds.y + item.bounds.height;
        if item_bottom > self.height {
            self.height = item_bottom;
        }
        self.items.push(item);
    }

    /// Serializes the display list to bytes (JSON format for cross-language compatibility).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Failed to serialize: {}", e))
    }

    /// Deserializes a display list from bytes (JSON format).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Failed to deserialize: {}", e))
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
