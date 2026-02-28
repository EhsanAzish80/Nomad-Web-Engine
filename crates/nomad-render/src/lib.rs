//! Rendering engine for the Nomad Web Engine.
//!
//! Converts layout boxes into a display list for cross-platform rendering.

use nomad_layout::{LayoutBox, LayoutContent};
use nomad_style::TextAlign;
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
    /// Interactive hit regions for user input
    pub hit_regions: Vec<HitRegion>,
}

impl DisplayList {
    /// Creates a new empty display list.
    pub fn new(width: f32) -> Self {
        Self {
            width,
            height: 0.0,
            items: Vec::new(),
            hit_regions: Vec::new(),
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

    /// Performs hit testing at the given point, returning the first matching hit region.
    /// 
    /// # Arguments
    /// * `x` - X coordinate in content space
    /// * `y` - Y coordinate in content space (including scroll offset)
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitRegion> {
        // Iterate in reverse order so top elements are tested first
        self.hit_regions.iter().rev().find(|region| region.rect.contains(x, y))
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
        /// Text alignment
        text_align: TextAlign,
    },
    /// Container box (for debugging/future use)
    Box {
        /// Background color (future)
        color: Option<String>,
    },
    /// Clickable link area (for anchor elements)
    Link {
        /// URL to navigate to
        url: String,
    },
    /// Text input field
    Input {
        /// Input name for form submission
        name: String,
        /// Input value/placeholder
        value: String,
        /// Input type (text, password, etc.)
        input_type: String,
    },
    /// Button (submit or button)
    Button {
        /// Button label
        label: String,
        /// Button type (submit, button, reset)
        button_type: String,
        /// Form ID this button belongs to
        form_id: Option<String>,
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

/// Type of interactive region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitRegionKind {
    /// Clickable link
    Link { url: String },
    /// Button (submit or regular)
    Button { 
        button_type: String,
        form_id: Option<String>,
    },
    /// Input field
    Input {
        control_id: String,
        name: String,
        input_type: String,
    },
}

/// An interactive region that can receive user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitRegion {
    /// Unique stable ID for this region
    pub id: String,
    /// Type and payload
    pub kind: HitRegionKind,
    /// Bounding rectangle
    pub rect: Rect,
}

/// Render engine that converts layout boxes to display list.
pub struct RenderEngine {
    viewport_width: f32,
    next_hit_region_id: usize,
}

impl RenderEngine {
    /// Creates a new render engine.
    pub fn new(viewport_width: f32) -> Self {
        Self { 
            viewport_width,
            next_hit_region_id: 0,
        }
    }

    /// Convert a layout box tree into a display list.
    pub fn render(&self, layout_root: &LayoutBox) -> Result<DisplayList, RenderError> {
        let mut display_list = DisplayList::new(self.viewport_width);
        let mut engine = Self {
            viewport_width: self.viewport_width,
            next_hit_region_id: 0,
        };

        // Recursively add display items
        engine.add_layout_box(&mut display_list, layout_root, 0.0, 0.0);

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
        &mut self,
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
                text_align,
            } => {
                if !content.trim().is_empty() {
                    display_list.items.push(DisplayItem {
                        kind: DisplayItemKind::Text {
                            content: content.clone(),
                            font_size: *font_size,
                            is_link: *is_link,
                            link_url: link_url.clone(),
                            text_align: *text_align,
                        },
                        bounds: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                    });
                }
            }
            LayoutContent::Input {
                name,
                value,
                input_type,
            } => {
                // Generate stable control ID
                let control_id = format!("input_{}", self.next_hit_region_id);
                self.next_hit_region_id += 1;
                
                display_list.items.push(DisplayItem {
                    kind: DisplayItemKind::Input {
                        name: name.clone(),
                        value: value.clone(),
                        input_type: input_type.clone(),
                    },
                    bounds: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                });
                
                // Add hit region for input
                display_list.hit_regions.push(HitRegion {
                    id: control_id.clone(),
                    kind: HitRegionKind::Input {
                        control_id,
                        name: name.clone(),
                        input_type: input_type.clone(),
                    },
                    rect: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                });
            }
            LayoutContent::Button {
                label,
                button_type,
                form_id,
            } => {
                // Generate stable region ID
                let region_id = format!("button_{}", self.next_hit_region_id);
                self.next_hit_region_id += 1;
                
                display_list.items.push(DisplayItem {
                    kind: DisplayItemKind::Button {
                        label: label.clone(),
                        button_type: button_type.clone(),
                        form_id: form_id.clone(),
                    },
                    bounds: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                });
                
                // Add hit region for button
                display_list.hit_regions.push(HitRegion {
                    id: region_id,
                    kind: HitRegionKind::Button {
                        button_type: button_type.clone(),
                        form_id: form_id.clone(),
                    },
                    rect: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                });
            }
            LayoutContent::Element { is_link, link_url, .. } => {
                // For link elements, create a clickable area
                if *is_link {
                    if let Some(url) = link_url {
                        // Generate stable region ID
                        let region_id = format!("link_{}", self.next_hit_region_id);
                        self.next_hit_region_id += 1;
                        
                        display_list.items.push(DisplayItem {
                            kind: DisplayItemKind::Link {
                                url: url.clone(),
                            },
                            bounds: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                        });
                        
                        // Add hit region for link
                        display_list.hit_regions.push(HitRegion {
                            id: region_id,
                            kind: HitRegionKind::Link {
                                url: url.clone(),
                            },
                            rect: Rect::new(abs_x, abs_y, layout_box.width, layout_box.height),
                        });
                    }
                }
                // For other elements, just process children
            }
            LayoutContent::Anonymous => {
                // For anonymous containers, just process children
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
                text_align: TextAlign::Left,
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
