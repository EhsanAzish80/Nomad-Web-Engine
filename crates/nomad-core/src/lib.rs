//! Core functionality for the Nomad Web Engine.
//!
//! This crate orchestrates all components: networking, HTML parsing,
//! CSS styling, layout computation, and rendering.

use nomad_html::{DomTree, HtmlError, HtmlParser};
use nomad_net::{NetworkError, NetworkLayer};
use nomad_style::StyleSheet;
use nomad_layout::{LayoutEngine, LayoutError};
use nomad_render::{RenderEngine, DisplayList, RenderError};
use markup5ever_rcdom::{Handle, NodeData};
use thiserror::Error;

// Re-export commonly used types
pub use nomad_render::{DisplayList as ExportedDisplayList, DisplayItem, DisplayItemKind, Rect};

/// Errors that can occur during engine operations.
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("HTML parsing error: {0}")]
    Html(#[from] HtmlError),

    #[error("Layout error: {0}")]
    Layout(#[from] LayoutError),

    #[error("Render error: {0}")]
    Render(#[from] RenderError),

    #[error("Engine not initialized")]
    NotInitialized,

    #[error("No content loaded")]
    NoContent,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Configuration for the engine.
pub struct EngineConfig {
    /// Maximum size of HTML response in bytes
    pub max_html_size: usize,
    /// Maximum number of DOM nodes
    pub max_dom_nodes: usize,
    /// Network timeout in seconds
    pub timeout_secs: u64,
    /// Viewport width
    pub viewport_width: f32,
    /// Viewport height
    pub viewport_height: f32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_html_size: nomad_net::MAX_RESPONSE_SIZE,
            max_dom_nodes: nomad_html::MAX_DOM_NODES,
            timeout_secs: nomad_net::DEFAULT_TIMEOUT_SECS,
            viewport_width: 800.0,
            viewport_height: 600.0,
        }
    }
}

/// The main engine struct that orchestrates the web engine components.
pub struct Engine {
    network: NetworkLayer,
    html_parser: HtmlParser,
    config: EngineConfig,
    current_dom: Option<DomTree>,
    current_stylesheet: StyleSheet,
    current_display_list: Option<DisplayList>,
}

impl Engine {
    /// Creates a new engine instance with default configuration.
    pub fn new() -> Result<Self, EngineError> {
        Self::with_config(EngineConfig::default())
    }

    /// Creates a new engine instance with custom configuration.
    pub fn with_config(config: EngineConfig) -> Result<Self, EngineError> {
        let network = NetworkLayer::with_config(config.timeout_secs, config.max_html_size)?;
        let html_parser = HtmlParser::with_max_nodes(config.max_dom_nodes);

        Ok(Self {
            network,
            html_parser,
            config,
            current_dom: None,
            current_stylesheet: StyleSheet::new(),
            current_display_list: None,
        })
    }

    /// Loads a URL and processes it through the full pipeline.
    ///
    /// This method:
    /// 1. Fetches the HTML from the URL
    /// 2. Parses it into a DOM tree
    /// 3. Extracts CSS from <style> tags
    /// 4. Computes layout with flexbox support
    /// 5. Generates a display list for rendering
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to load
    ///
    /// # Returns
    ///
    /// A string containing the extracted text content.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - HTML parsing fails
    /// - Layout computation fails
    /// - Resource limits are exceeded
    pub fn load_url(&mut self, url: &str) -> Result<String, EngineError> {
        // Fetch HTML
        let response = self.network.fetch(url)?;

        // Parse HTML into DOM
        let dom = self.html_parser.parse(&response.body)?;

        // Extract text for return value
        let text = dom.extract_text();

        // Extract CSS from <style> tags
        let css = extract_css_from_dom(dom.document());
        self.current_stylesheet = StyleSheet::parse(&css);

        // Store the DOM
        self.current_dom = Some(dom);

        // Generate display list
        self.regenerate_display_list()?;

        Ok(text)
    }

    /// Regenerates the display list from the current DOM and styles.
    pub fn regenerate_display_list(&mut self) -> Result<(), EngineError> {
        if let Some(ref dom) = self.current_dom {
            // Create layout engine
            let mut layout_engine = LayoutEngine::new();

            // Compute layout
            let layout_box = layout_engine.compute_layout(
                dom.document(),
                &self.current_stylesheet,
                self.config.viewport_width,
                self.config.viewport_height,
            )?;

            // Create render engine and generate display list
            let render_engine = RenderEngine::new(self.config.viewport_width);
            let display_list = render_engine.render(&layout_box)?;

            self.current_display_list = Some(display_list);
            Ok(())
        } else {
            Err(EngineError::NoContent)
        }
    }

    /// Returns the current display list as bytes.
    pub fn get_display_list_bytes(&self) -> Result<Vec<u8>, EngineError> {
        if let Some(ref display_list) = self.current_display_list {
            Ok(display_list.to_bytes()?)
        } else {
            Err(EngineError::NoContent)
        }
    }

    /// Returns a reference to the current display list.
    pub fn get_display_list(&self) -> Option<&DisplayList> {
        self.current_display_list.as_ref()
    }

    /// Sets the viewport dimensions and regenerates layout.
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.config.viewport_width = width;
        self.config.viewport_height = height;
        
        // Regenerate layout if we have content
        let _ = self.regenerate_display_list();
    }

    /// Sets the viewport width for layout.
    pub fn set_viewport_width(&mut self, width: f32) {
        self.set_viewport_size(width, self.config.viewport_height);
    }

    /// Loads a URL and prints the extracted text to stdout.
    ///
    /// This is a convenience method that combines `load_url` with printing.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to load
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails.
    pub fn load_url_and_print(&mut self, url: &str) -> Result<(), EngineError> {
        let text = self.load_url(url)?;
        println!("{}", text);
        Ok(())
    }

    /// Returns the engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new().expect("Failed to create default Engine")
    }
}

/// Extract CSS content from <style> tags in the DOM.
fn extract_css_from_dom(handle: &Handle) -> String {
    let mut css = String::new();
    extract_css_recursive(handle, &mut css);
    css
}

/// Recursively extract CSS from <style> tags.
fn extract_css_recursive(handle: &Handle, css: &mut String) {
    match &handle.data {
        NodeData::Element { ref name, .. } => {
            let tag_name = name.local.as_ref();
            
            if tag_name == "style" {
                // Extract text content from this style tag
                for child in handle.children.borrow().iter() {
                    if let NodeData::Text { ref contents } = child.data {
                        css.push_str(&contents.borrow().to_string());
                        css.push('\n');
                    }
                }
            } else {
                // Continue searching in children
                for child in handle.children.borrow().iter() {
                    extract_css_recursive(child, css);
                }
            }
        }
        _ => {
            // For non-element nodes, search children
            for child in handle.children.borrow().iter() {
                extract_css_recursive(child, css);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_with_custom_config() {
        let config = EngineConfig {
            max_html_size: 1024 * 1024,
            max_dom_nodes: 50_000,
            timeout_secs: 15,
            viewport_width: 1024.0,
            viewport_height: 768.0,
        };
        let engine = Engine::with_config(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.max_html_size, nomad_net::MAX_RESPONSE_SIZE);
        assert_eq!(config.max_dom_nodes, nomad_html::MAX_DOM_NODES);
        assert_eq!(config.timeout_secs, nomad_net::DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.viewport_width, 800.0);
        assert_eq!(config.viewport_height, 600.0);
    }

    #[test]
    fn test_css_extraction() {
        // Create a simple DOM for testing
        use markup5ever_rcdom::{Node, NodeData};
        use std::rc::Rc;
        use std::cell::{RefCell, Cell};

        let node = Rc::new(Node {
            parent: Cell::new(None),
            children: RefCell::new(vec![]),
            data: NodeData::Document,
        });

        let css = extract_css_from_dom(&node);
        assert_eq!(css, "");
    }
}
