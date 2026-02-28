//! Layout engine for the Nomad Web Engine.
//!
//! Computes layout and positions elements based on CSS box model and flexbox.
//! Uses Taffy for layout computation with strict limits for safety.

use nomad_style::{ComputedStyle, Display, FlexDirection, JustifyContent, AlignItems, EdgeInsets, StyleSheet};
use markup5ever_rcdom::{Handle, NodeData};
use taffy::{
    prelude::*,
    style::{Dimension, LengthPercentage, LengthPercentageAuto},
    TaffyTree,
};
use thiserror::Error;

/// Safety limits to prevent pathological layouts.
pub const MAX_DOM_DEPTH: usize = 100;
pub const MAX_LAYOUT_NODES: usize = 10_000;
pub const MAX_LAYOUT_PASSES: usize = 3;

/// Errors that can occur during layout.
#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("Layout depth exceeded maximum of {}", MAX_DOM_DEPTH)]
    DepthExceeded,

    #[error("Layout node count exceeded maximum of {}", MAX_LAYOUT_NODES)]
    NodeCountExceeded,

    #[error("Taffy layout error: {0}")]
    TaffyError(String),
}

/// A positioned layout box.
#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub content: LayoutContent,
    pub children: Vec<LayoutBox>,
}

/// Content type for a layout box.
#[derive(Debug, Clone)]
pub enum LayoutContent {
    Element {
        tag_name: String,
        is_link: bool,
        link_url: Option<String>,
    },
    Text {
        content: String,
        font_size: f32,
        is_link: bool,
        link_url: Option<String>,
    },
    Input {
        name: String,
        value: String,
        input_type: String,
    },
    Button {
        label: String,
        button_type: String,
        form_id: Option<String>,
    },
    Anonymous, // For containers without specific content
}

/// Layout engine that computes positions using Taffy.
pub struct LayoutEngine {
    taffy: TaffyTree<()>,
    node_counter: usize,
    depth_counter: usize,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_counter: 0,
            depth_counter: 0,
        }
    }

    /// Compute layout for a DOM tree with given styles.
    pub fn compute_layout(
        &mut self,
        root: &Handle,
        stylesheet: &StyleSheet,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Result<LayoutBox, LayoutError> {
        // Reset counters
        self.node_counter = 0;
        self.depth_counter = 0;

        // Build taffy tree from DOM
        let taffy_node = self.build_taffy_tree(root, stylesheet, None, 0)?;

        // Compute layout
        self.taffy
            .compute_layout(
                taffy_node,
                Size {
                    width: AvailableSpace::Definite(viewport_width),
                    height: AvailableSpace::Definite(viewport_height),
                },
            )
            .map_err(|e| LayoutError::TaffyError(format!("{:?}", e)))?;

        // Extract layout information
        let layout_box = self.extract_layout(taffy_node, root, stylesheet, None)?;

        Ok(layout_box)
    }

    /// Build a taffy node tree from a DOM node.
    fn build_taffy_tree(
        &mut self,
        node: &Handle,
        stylesheet: &StyleSheet,
        parent_style: Option<&ComputedStyle>,
        depth: usize,
    ) -> Result<taffy::prelude::NodeId, LayoutError> {
        // Check depth limit
        if depth > MAX_DOM_DEPTH {
            return Err(LayoutError::DepthExceeded);
        }

        // Check node count limit
        if self.node_counter > MAX_LAYOUT_NODES {
            return Err(LayoutError::NodeCountExceeded);
        }
        self.node_counter += 1;

        // Compute style for this node
        let style = stylesheet.compute_style(node, parent_style);

        // Skip if display: none
        if style.display == Display::None {
            // Return a zero-sized node
            return self.taffy.new_leaf(Style::default())
                .map_err(|e| LayoutError::TaffyError(format!("{:?}", e)));
        }

        // Process children
        let mut child_nodes = Vec::new();
        for child in node.children.borrow().iter() {
            match &child.data {
                NodeData::Text { ref contents } => {
                    // For text nodes, create a leaf with content size
                    let text = contents.borrow().to_string().trim().to_string();
                    if !text.is_empty() {
                        let text_node = self.create_text_node(&text, &style)?;
                        child_nodes.push(text_node);
                    }
                }
                NodeData::Element { ref name, .. } => {
                    let tag_name = name.local.as_ref();
                    
                    // Skip invisible elements
                    if matches!(
                        tag_name,
                        "script" | "style" | "noscript" | "iframe" | "head"
                    ) {
                        continue;
                    }

                    // Recursively build child
                    let child_node = self.build_taffy_tree(child, stylesheet, Some(&style), depth + 1)?;
                    child_nodes.push(child_node);
                }
                _ => {
                    // For other node types, process their children
                    for grandchild in child.children.borrow().iter() {
                        let grandchild_node = self.build_taffy_tree(grandchild, stylesheet, Some(&style), depth + 1)?;
                        child_nodes.push(grandchild_node);
                    }
                }
            }
        }

        // Convert ComputedStyle to Taffy Style
        let taffy_style = self.computed_style_to_taffy(&style);

        // Create node with children
        self.taffy
            .new_with_children(taffy_style, &child_nodes)
            .map_err(|e| LayoutError::TaffyError(format!("{:?}", e)))
    }

    /// Create a text node with estimated size.
    fn create_text_node(&mut self, text: &str, style: &ComputedStyle) -> Result<taffy::prelude::NodeId, LayoutError> {
        // Estimate text dimensions (rough approximation)
        let char_width = style.font_size * 0.6; // Average character width
        let text_width = text.len() as f32 * char_width;
        let text_height = style.font_size * 1.2; // Line height

        let text_style = Style {
            size: Size {
                width: Dimension::Length(text_width),
                height: Dimension::Length(text_height),
            },
            ..Default::default()
        };

        self.taffy
            .new_leaf(text_style)
            .map_err(|e| LayoutError::TaffyError(format!("{:?}", e)))
    }

    /// Convert ComputedStyle to Taffy Style.
    fn computed_style_to_taffy(&self, style: &ComputedStyle) -> Style {
        let display = match style.display {
            Display::Block => taffy::style::Display::Block,
            Display::Inline => taffy::style::Display::Block, // Treat inline as block for now
            Display::Flex => taffy::style::Display::Flex,
            Display::None => taffy::style::Display::None,
        };

        let flex_direction = match style.flex_direction {
            FlexDirection::Row => taffy::style::FlexDirection::Row,
            FlexDirection::Column => taffy::style::FlexDirection::Column,
        };

        let justify_content = match style.justify_content {
            JustifyContent::FlexStart => Some(taffy::style::JustifyContent::FlexStart),
            JustifyContent::FlexEnd => Some(taffy::style::JustifyContent::FlexEnd),
            JustifyContent::Center => Some(taffy::style::JustifyContent::Center),
            JustifyContent::SpaceBetween => Some(taffy::style::JustifyContent::SpaceBetween),
            JustifyContent::SpaceAround => Some(taffy::style::JustifyContent::SpaceAround),
        };

        let align_items = match style.align_items {
            AlignItems::Stretch => Some(taffy::style::AlignItems::Stretch),
            AlignItems::FlexStart => Some(taffy::style::AlignItems::FlexStart),
            AlignItems::FlexEnd => Some(taffy::style::AlignItems::FlexEnd),
            AlignItems::Center => Some(taffy::style::AlignItems::Center),
        };

        Style {
            display,
            flex_direction,
            justify_content,
            align_items,
            margin: self.edge_insets_to_margin(style.margin),
            padding: self.edge_insets_to_padding(style.padding),
            size: Size {
                width: style.width.map(Dimension::Length).unwrap_or(Dimension::Auto),
                height: style.height.map(Dimension::Length).unwrap_or(Dimension::Auto),
            },
            ..Default::default()
        }
    }

    /// Convert EdgeInsets to Taffy Rect for margin (LengthPercentageAuto).
    fn edge_insets_to_margin(&self, insets: EdgeInsets) -> Rect<LengthPercentageAuto> {
        Rect {
            left: LengthPercentageAuto::Length(insets.left),
            right: LengthPercentageAuto::Length(insets.right),
            top: LengthPercentageAuto::Length(insets.top),
            bottom: LengthPercentageAuto::Length(insets.bottom),
        }
    }

    /// Convert EdgeInsets to Taffy Rect for padding (LengthPercentage).
    fn edge_insets_to_padding(&self, insets: EdgeInsets) -> Rect<LengthPercentage> {
        Rect {
            left: LengthPercentage::Length(insets.left),
            right: LengthPercentage::Length(insets.right),
            top: LengthPercentage::Length(insets.top),
            bottom: LengthPercentage::Length(insets.bottom),
        }
    }

    /// Extract layout information from computed taffy layout.
    fn extract_layout(
        &self,
        taffy_node: taffy::prelude::NodeId,
        dom_node: &Handle,
        stylesheet: &StyleSheet,
        parent_style: Option<&ComputedStyle>,
    ) -> Result<LayoutBox, LayoutError> {
        let layout = self.taffy.layout(taffy_node)
            .map_err(|e| LayoutError::TaffyError(format!("{:?}", e)))?;

        let style = stylesheet.compute_style(dom_node, parent_style);

        // Determine content type
        let content = match &dom_node.data {
            NodeData::Text { ref contents } => {
                let text = contents.borrow().to_string().trim().to_string();
                LayoutContent::Text {
                    content: text,
                    font_size: style.font_size,
                    is_link: false,
                    link_url: None,
                }
            }
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag_name = name.local.as_ref().to_string();
                
                // Helper to get attribute value
                let get_attr = |attr_name: &str| -> Option<String> {
                    attrs.borrow().iter()
                        .find(|a| a.name.local.as_ref() == attr_name)
                        .map(|a| a.value.to_string())
                };

                // Handle input elements
                if tag_name == "input" {
                    let name = get_attr("name").unwrap_or_default();
                    let value = get_attr("value").unwrap_or_default();
                    let input_type = get_attr("type").unwrap_or_else(|| "text".to_string());
                    
                    LayoutContent::Input {
                        name,
                        value,
                        input_type,
                    }
                }
                // Handle button elements
                else if tag_name == "button" {
                    let button_type = get_attr("type").unwrap_or_else(|| "button".to_string());
                    let form_id = get_attr("form");
                    
                    // Extract text content for label
                    let label = extract_text_content(dom_node);
                    
                    LayoutContent::Button {
                        label,
                        button_type,
                        form_id,
                    }
                }
                // Handle input type="submit" as button
                else if tag_name == "input" && get_attr("type").as_deref() == Some("submit") {
                    let label = get_attr("value").unwrap_or_else(|| "Submit".to_string());
                    let form_id = get_attr("form");
                    
                    LayoutContent::Button {
                        label,
                        button_type: "submit".to_string(),
                        form_id,
                    }
                }
                // Handle anchor links
                else if tag_name == "a" {
                    let link_url = get_attr("href");
                    
                    LayoutContent::Element {
                        tag_name,
                        is_link: true,
                        link_url,
                    }
                }
                // Default element handling
                else {
                    LayoutContent::Element {
                        tag_name,
                        is_link: false,
                        link_url: None,
                    }
                }
            }
            _ => LayoutContent::Anonymous,
        };

        // Extract children layouts - must match the same filtering logic as build_taffy_tree
        let mut children = Vec::new();
        let taffy_children = self.taffy.children(taffy_node)
            .map_err(|e| LayoutError::TaffyError(format!("{:?}", e)))?;

        let mut taffy_child_idx = 0;
        
        // Iterate through DOM children using the same logic as build_taffy_tree
        for child in dom_node.children.borrow().iter() {
            if taffy_child_idx >= taffy_children.len() {
                break;
            }

            match &child.data {
                NodeData::Text { ref contents } => {
                    let text = contents.borrow().to_string().trim().to_string();
                    if !text.is_empty() {
                        // This text node has a corresponding taffy node
                        let child_layout = self.extract_layout(
                            taffy_children[taffy_child_idx],
                            child,
                            stylesheet,
                            Some(&style)
                        )?;
                        children.push(child_layout);
                        taffy_child_idx += 1;
                    }
                }
                NodeData::Element { ref name, .. } => {
                    let tag_name = name.local.as_ref();
                    
                    // Skip invisible elements (same as build_taffy_tree)
                    if matches!(
                        tag_name,
                        "script" | "style" | "noscript" | "iframe" | "head"
                    ) {
                        continue;
                    }

                    // This element has a corresponding taffy node
                    let child_layout = self.extract_layout(
                        taffy_children[taffy_child_idx],
                        child,
                        stylesheet,
                        Some(&style)
                    )?;
                    children.push(child_layout);
                    taffy_child_idx += 1;
                }
                _ => {
                    // For other node types, process their grandchildren
                    for grandchild in child.children.borrow().iter() {
                        if taffy_child_idx >= taffy_children.len() {
                            break;
                        }
                        let grandchild_layout = self.extract_layout(
                            taffy_children[taffy_child_idx],
                            grandchild,
                            stylesheet,
                            Some(&style)
                        )?;
                        children.push(grandchild_layout);
                        taffy_child_idx += 1;
                    }
                }
            }
        }

        Ok(LayoutBox {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
            content,
            children,
        })
    }
}

/// Helper function to extract text content from a node and its children
fn extract_text_content(node: &Handle) -> String {
    let mut text = String::new();
    extract_text_recursive(node, &mut text);
    text.trim().to_string()
}

fn extract_text_recursive(node: &Handle, acc: &mut String) {
    match &node.data {
        NodeData::Text { ref contents } => {
            acc.push_str(&contents.borrow());
        }
        _ => {
            for child in node.children.borrow().iter() {
                extract_text_recursive(child, acc);
            }
        }
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_engine_creation() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.node_counter, 0);
    }

    #[test]
    fn test_depth_limit() {
        assert!(MAX_DOM_DEPTH > 0);
        assert!(MAX_DOM_DEPTH <= 100);
    }

    #[test]
    fn test_node_count_limit() {
        assert!(MAX_LAYOUT_NODES > 0);
        assert!(MAX_LAYOUT_NODES <= 10_000);
    }
}
