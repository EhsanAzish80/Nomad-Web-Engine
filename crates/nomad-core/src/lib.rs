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

/// Form metadata extracted from the DOM
#[derive(Debug, Clone)]
pub struct FormMetadata {
    /// Form ID (if present)
    pub id: Option<String>,
    /// Form action URL (where to submit)
    pub action: String,
    /// Form method (GET or POST, we only support GET)
    pub method: String,
}

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
    current_forms: Vec<FormMetadata>,
    current_url: Option<String>,
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
            current_forms: Vec::new(),
            current_url: None,
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

        // Extract inline CSS from <style> tags
        let inline_css = extract_css_from_dom(dom.document());
        
        // Extract external CSS from <link rel="stylesheet"> tags (same-origin only)
        let css_urls = extract_external_css_urls(dom.document(), url);
        
        // Fetch external CSS files
        let mut external_css = String::new();
        for css_url in css_urls {
            // Resolve relative URLs
            let absolute_url = if css_url.starts_with("http://") || css_url.starts_with("https://") {
                css_url
            } else if css_url.starts_with('/') {
                // Absolute path relative to origin
                let origin = get_origin(url);
                format!("{}{}", origin, css_url)
            } else {
                // Relative path
                if let Some(last_slash) = url.rfind('/') {
                    let base = &url[..last_slash + 1];
                    format!("{}{}", base, css_url)
                } else {
                    css_url
                }
            };
            
            // Fetch CSS with size limit
            match self.network.fetch(&absolute_url) {
                Ok(response) => {
                    // Enforce MAX_CSS_FILE_SIZE limit (1MB)
                    if response.body.len() <= nomad_style::MAX_CSS_FILE_SIZE {
                        external_css.push_str(&response.body);
                        external_css.push('\n');
                    }
                }
                Err(_) => {
                    // Ignore CSS fetch errors, continue with what we have
                }
            }
        }
        
        // Merge CSS: external first, then inline (per CSS cascade rules)
        let mut combined_css = external_css;
        combined_css.push_str(&inline_css);
        
        // Parse stylesheet with automatic rule limiting
        self.current_stylesheet = StyleSheet::parse(&combined_css);

        // Extract form metadata
        self.current_forms = extract_forms_from_dom(dom.document(), url);

        // Store current URL for relative URL resolution
        self.current_url = Some(url.to_string());

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

    /// Submits a GET form with the given input values.
    ///
    /// This builds a query string from the inputs and navigates to the action URL.
    ///
    /// # Arguments
    ///
    /// * `form_index` - Index of the form in current_forms (0 for first form)
    /// * `inputs` - Map of input name to value
    ///
    /// # Returns
    ///
    /// Returns the extracted text from the resulting page.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Form index is out of bounds
    /// - Form method is not GET
    /// - Navigation fails
    pub fn submit_form(&mut self, form_index: usize, inputs: &[(String, String)]) -> Result<String, EngineError> {
        // Get the form metadata
        let form = self.current_forms.get(form_index)
            .ok_or(EngineError::NoContent)?
            .clone();

        // Only support GET forms
        if form.method != "GET" {
            return Err(EngineError::Serialization(
                format!("Only GET forms are supported, got method: {}", form.method)
            ));
        }

        // Build query string
        let query_string = build_query_string(inputs);

        // Build final URL
        let url = if form.action.contains('?') {
            format!("{}&{}", form.action, query_string)
        } else {
            format!("{}?{}", form.action, query_string)
        };

        // Load the URL
        self.load_url(&url)
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

/// Extract external CSS stylesheet URLs from <link rel="stylesheet"> tags.
/// Returns URLs that are same-origin as the base URL.
fn extract_external_css_urls(handle: &Handle, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    
    // Parse base URL to get origin
    let base_origin = get_origin(base_url);
    
    extract_css_links_recursive(handle, &mut urls, &base_origin);
    
    // Enforce MAX_CSS_FILES limit
    urls.truncate(nomad_style::MAX_CSS_FILES);
    
    urls
}

/// Recursively extract stylesheet URLs from <link> tags.
fn extract_css_links_recursive(handle: &Handle, urls: &mut Vec<String>, base_origin: &str) {
    match &handle.data {
        NodeData::Element { ref name, ref attrs, .. } => {
            let tag_name = name.local.as_ref();
            
            if tag_name == "link" {
                let attrs = attrs.borrow();
                
                // Check if rel="stylesheet"
                let is_stylesheet = attrs.iter()
                    .any(|a| a.name.local.as_ref() == "rel" && a.value.to_string() == "stylesheet");
                
                if is_stylesheet {
                    // Get href attribute
                    if let Some(href_attr) = attrs.iter().find(|a| a.name.local.as_ref() == "href") {
                        let href = href_attr.value.to_string();
                        
                        // Check same-origin (only allow same origin for security)
                        if is_same_origin(&href, base_origin) {
                            urls.push(href);
                        }
                    }
                }
            }
            
            // Continue searching in children
            for child in handle.children.borrow().iter() {
                extract_css_links_recursive(child, urls, base_origin);
            }
        }
        _ => {
            // For non-element nodes, search children
            for child in handle.children.borrow().iter() {
                extract_css_links_recursive(child, urls, base_origin);
            }
        }
    }
}

/// Get the origin (scheme + host + port) from a URL.
fn get_origin(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let after_scheme = &url[scheme_end + 3..];
        if let Some(path_start) = after_scheme.find('/') {
            let host = &after_scheme[..path_start];
            return format!("{}://{}", &url[..scheme_end], host);
        } else {
            return format!("{}://{}", &url[..scheme_end], after_scheme);
        }
    }
    String::new()
}

/// Check if a URL is same-origin or relative.
fn is_same_origin(href: &str, base_origin: &str) -> bool {
    // Relative URLs are considered same-origin
    if !href.starts_with("http://") && !href.starts_with("https://") {
        return true;
    }
    
    // Absolute URLs must match origin
    let href_origin = get_origin(href);
    href_origin == base_origin
}

/// Extract form metadata from the DOM.
fn extract_forms_from_dom(handle: &Handle, base_url: &str) -> Vec<FormMetadata> {
    let mut forms = Vec::new();
    extract_forms_recursive(handle, base_url, &mut forms);
    forms
}

/// Recursively extract form metadata from <form> tags.
fn extract_forms_recursive(handle: &Handle, base_url: &str, forms: &mut Vec<FormMetadata>) {
    match &handle.data {
        NodeData::Element { ref name, ref attrs, .. } => {
            let tag_name = name.local.as_ref();
            
            if tag_name == "form" {
                let attrs = attrs.borrow();
                
                // Get form attributes
                let id = attrs.iter()
                    .find(|a| a.name.local.as_ref() == "id")
                    .map(|a| a.value.to_string());
                
                let action = attrs.iter()
                    .find(|a| a.name.local.as_ref() == "action")
                    .map(|a| a.value.to_string())
                    .unwrap_or_else(|| base_url.to_string());
                
                let method = attrs.iter()
                    .find(|a| a.name.local.as_ref() == "method")
                    .map(|a| a.value.to_string().to_uppercase())
                    .unwrap_or_else(|| "GET".to_string());
                
                forms.push(FormMetadata {
                    id,
                    action,
                    method,
                });
            }
            
            // Continue searching in children
            for child in handle.children.borrow().iter() {
                extract_forms_recursive(child, base_url, forms);
            }
        }
        _ => {
            // For non-element nodes, search children
            for child in handle.children.borrow().iter() {
                extract_forms_recursive(child, base_url, forms);
            }
        }
    }
}

/// Build a URL query string from input name-value pairs.
fn build_query_string(inputs: &[(String, String)]) -> String {
    inputs.iter()
        .map(|(name, value)| {
            format!("{}={}", 
                urlencoding::encode(name), 
                urlencoding::encode(value))
        })
        .collect::<Vec<_>>()
        .join("&")
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
