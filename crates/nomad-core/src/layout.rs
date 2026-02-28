//! Basic text layout engine.
//!
//! Performs simple text layout from HTML DOM, creating a display list.

use crate::display_list::{DisplayItem, DisplayItemKind, DisplayList, Rect};
use markup5ever_rcdom::{Handle, NodeData};

/// Configuration for layout.
pub struct LayoutConfig {
    /// Viewport width
    pub viewport_width: f32,
    /// Left padding
    pub padding_left: f32,
    /// Right padding
    pub padding_right: f32,
    /// Top padding
    pub padding_top: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Default font size
    pub font_size: f32,
    /// Maximum text width per line (approximate)
    pub max_line_width: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            viewport_width: 800.0,
            padding_left: 20.0,
            padding_right: 20.0,
            padding_top: 20.0,
            line_height: 1.5,
            font_size: 16.0,
            max_line_width: 760.0, // viewport_width - padding_left - padding_right
        }
    }
}

/// Layout context that tracks state during layout.
struct LayoutContext {
    config: LayoutConfig,
    current_y: f32,
    current_x: f32,
    current_link_url: Option<String>,
}

impl LayoutContext {
    fn new(config: LayoutConfig) -> Self {
        let current_y = config.padding_top;
        let current_x = config.padding_left;
        Self {
            config,
            current_y,
            current_x,
            current_link_url: None,
        }
    }

    fn line_height(&self) -> f32 {
        self.config.font_size * self.config.line_height
    }

    fn advance_line(&mut self) {
        self.current_y += self.line_height();
        self.current_x = self.config.padding_left;
    }
}

/// Performs layout on a DOM tree and generates a display list.
pub fn layout_dom(handle: &Handle, config: LayoutConfig) -> DisplayList {
    let mut display_list = DisplayList::new(config.viewport_width);
    let mut context = LayoutContext::new(config);

    layout_node(handle, &mut display_list, &mut context);

    display_list
}

/// Recursively layouts a DOM node.
fn layout_node(handle: &Handle, display_list: &mut DisplayList, context: &mut LayoutContext) {
    match handle.data {
        NodeData::Text { ref contents } => {
            let text = contents.borrow().to_string();
            layout_text(&text, display_list, context);
        }
        NodeData::Element { ref name, .. } => {
            let tag_name = name.local.as_ref();

            // Skip non-visible elements
            if matches!(
                tag_name,
                "script" | "style" | "noscript" | "iframe" | "object" | "embed" | "head"
            ) {
                return;
            }

            // Check if this is a link
            let is_link = tag_name == "a";
            let old_link_url = context.current_link_url.clone();

            if is_link {
                // Extract href attribute
                if let NodeData::Element { ref attrs, .. } = handle.data {
                    for attr in attrs.borrow().iter() {
                        if attr.name.local.as_ref() == "href" {
                            context.current_link_url = Some(attr.value.to_string());
                            break;
                        }
                    }
                }
            }

            // Add spacing for block elements
            if matches!(tag_name, "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "br") {
                if context.current_x > context.config.padding_left {
                    context.advance_line();
                }
            }

            // Process children
            for child in handle.children.borrow().iter() {
                layout_node(child, display_list, context);
            }

            // Add spacing after block elements
            if matches!(tag_name, "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                context.advance_line();
            }

            // Restore link state
            if is_link {
                context.current_link_url = old_link_url;
            }
        }
        _ => {
            // For Document, Doctype, Comment, etc., process children
            for child in handle.children.borrow().iter() {
                layout_node(child, display_list, context);
            }
        }
    }
}

/// Layouts text content, handling word wrapping.
fn layout_text(text: &str, display_list: &mut DisplayList, context: &mut LayoutContext) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    // Simple word-based wrapping
    let words: Vec<&str> = trimmed.split_whitespace().collect();

    for word in words {
        // Approximate character width (very rough)
        let char_width = context.config.font_size * 0.6;
        let word_width = word.len() as f32 * char_width;

        // Check if we need to wrap
        if context.current_x + word_width
            > context.config.padding_left + context.config.max_line_width
            && context.current_x > context.config.padding_left
        {
            context.advance_line();
        }

        // Add the word
        let item = DisplayItem {
            kind: DisplayItemKind::Text {
                content: word.to_string() + " ",
                font_size: context.config.font_size,
                is_link: context.current_link_url.is_some(),
                link_url: context.current_link_url.clone(),
            },
            bounds: Rect::new(
                context.current_x,
                context.current_y,
                word_width + char_width, // Add space width
                context.config.font_size,
            ),
        };

        display_list.add_item(item);
        context.current_x += word_width + char_width;
    }
}
