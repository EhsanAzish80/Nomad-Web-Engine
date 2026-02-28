//! Core functionality for the Nomad Web Engine.
//!
//! This crate provides foundational types and utilities used across
//! the entire web engine.

use nomad_html::{DomTree, HtmlError, HtmlParser};
use nomad_net::{NetworkError, NetworkLayer};
use thiserror::Error;

pub mod display_list;
pub mod layout;

pub use display_list::{DisplayItem, DisplayItemKind, DisplayList, Rect};
pub use layout::{layout_dom, LayoutConfig};

/// Errors that can occur during engine operations.
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("HTML parsing error: {0}")]
    Html(#[from] HtmlError),

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
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_html_size: nomad_net::MAX_RESPONSE_SIZE,
            max_dom_nodes: nomad_html::MAX_DOM_NODES,
            timeout_secs: nomad_net::DEFAULT_TIMEOUT_SECS,
        }
    }
}

/// The main engine struct that orchestrates the web engine components.
pub struct Engine {
    network: NetworkLayer,
    html_parser: HtmlParser,
    config: EngineConfig,
    layout_config: LayoutConfig,
    current_dom: Option<DomTree>,
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
            layout_config: LayoutConfig::default(),
            current_dom: None,
            current_display_list: None,
        })
    }

    /// Loads a URL and returns the extracted text content.
    ///
    /// This method:
    /// 1. Fetches the HTML from the URL
    /// 2. Parses it into a DOM tree
    /// 3. Extracts visible text nodes
    /// 4. Returns the cleaned text
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
    /// - Resource limits are exceeded
    pub fn load_url(&mut self, url: &str) -> Result<String, EngineError> {
        // Fetch HTML
        let response = self.network.fetch(url)?;

        // Parse HTML into DOM
        let dom = self.html_parser.parse(&response.body)?;

        // Store the DOM
        let text = dom.extract_text();
        self.current_dom = Some(dom);

        // Generate initial display list
        self.regenerate_display_list()?;

        // Extract and return text
        Ok(text)
    }

    /// Regenerates the display list from the current DOM.
    pub fn regenerate_display_list(&mut self) -> Result<(), EngineError> {
        if let Some(ref dom) = self.current_dom {
            let display_list = layout_dom(dom.document(), self.layout_config.clone());
            self.current_display_list = Some(display_list);
            Ok(())
        } else {
            Err(EngineError::NoContent)
        }
    }

    /// Returns the current display list as bytes.
    pub fn get_display_list_bytes(&self) -> Result<Vec<u8>, EngineError> {
        if let Some(ref display_list) = self.current_display_list {
            display_list
                .to_bytes()
                .map_err(EngineError::Serialization)
        } else {
            Err(EngineError::NoContent)
        }
    }

    /// Returns a reference to the current display list.
    pub fn get_display_list(&self) -> Option<&DisplayList> {
        self.current_display_list.as_ref()
    }

    /// Ticks the engine (for future animation/updates).
    pub fn tick(&mut self) {
        // Placeholder for future updates
    }

    /// Sets the viewport width for layout.
    pub fn set_viewport_width(&mut self, width: f32) {
        self.layout_config.viewport_width = width;
        self.layout_config.max_line_width = width - self.layout_config.padding_left - self.layout_config.padding_right;
        // Regenerate layout if we have content
        let _ = self.regenerate_display_list();
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

/// Legacy core placeholder struct (kept for backward compatibility).
#[deprecated(since = "0.1.0", note = "Use Engine instead")]
pub struct NomadCore {
    initialized: bool,
}

#[allow(deprecated)]
impl NomadCore {
    /// Creates a new instance of the core engine.
    pub fn new() -> Self {
        Self { initialized: true }
    }

    /// Returns whether the core is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

#[allow(deprecated)]
impl Default for NomadCore {
    fn default() -> Self {
        Self::new()
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
    }

    #[test]
    #[allow(deprecated)]
    fn test_legacy_core() {
        let core = NomadCore::new();
        assert!(core.is_initialized());
    }
}
