//! HTML parsing and DOM representation for the Nomad Web Engine.
//!
//! Provides HTML5 parsing, DOM tree construction, and manipulation.

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::default::Default;
use thiserror::Error;

/// Maximum number of DOM nodes allowed
pub const MAX_DOM_NODES: usize = 100_000;

/// HTML parsing errors.
#[derive(Error, Debug)]
pub enum HtmlError {
    #[error("DOM tree exceeds maximum node limit ({MAX_DOM_NODES} nodes)")]
    TooManyNodes,

    #[error("Failed to parse HTML: {0}")]
    ParseError(String),
}

/// HTML parser for converting HTML text into a DOM tree.
pub struct HtmlParser {
    max_nodes: usize,
}

impl HtmlParser {
    /// Creates a new HTML parser with default settings.
    pub fn new() -> Self {
        Self {
            max_nodes: MAX_DOM_NODES,
        }
    }

    /// Creates a new HTML parser with a custom node limit.
    pub fn with_max_nodes(max_nodes: usize) -> Self {
        Self { max_nodes }
    }

    /// Parses HTML content into a DOM tree.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to parse
    ///
    /// # Returns
    ///
    /// A `DomTree` representing the parsed document.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The DOM tree exceeds the maximum node limit
    pub fn parse(&self, html: &str) -> Result<DomTree, HtmlError> {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .map_err(|e| HtmlError::ParseError(e.to_string()))?;

        let node_count = count_nodes(&dom.document);

        if node_count > self.max_nodes {
            return Err(HtmlError::TooManyNodes);
        }

        Ok(DomTree {
            document: dom.document,
            node_count,
        })
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// DOM tree representation.
pub struct DomTree {
    document: Handle,
    node_count: usize,
}

impl DomTree {
    /// Returns the number of nodes in the DOM tree.
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Extracts all visible text from the DOM tree.
    ///
    /// This traverses the tree and collects text from text nodes,
    /// ignoring script, style, and other non-visible elements.
    pub fn extract_text(&self) -> String {
        let mut text = Vec::new();
        extract_text_recursive(&self.document, &mut text);

        // Join with spaces and clean up whitespace
        text.join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns a reference to the document root.
    pub fn document(&self) -> &Handle {
        &self.document
    }
}

/// Recursively counts nodes in the DOM tree.
fn count_nodes(handle: &Handle) -> usize {
    let mut count = 1;
    for child in handle.children.borrow().iter() {
        count += count_nodes(child);
    }
    count
}

/// Recursively extracts text from nodes, filtering out non-visible content.
fn extract_text_recursive(handle: &Handle, text: &mut Vec<String>) {
    match handle.data {
        NodeData::Text { ref contents } => {
            let content = contents.borrow().to_string();
            // Only add non-empty text
            if !content.trim().is_empty() {
                text.push(content.trim().to_string());
            }
        }
        NodeData::Element { ref name, .. } => {
            let tag_name = name.local.as_ref();

            // Skip script, style, and other non-visible elements
            if matches!(
                tag_name,
                "script" | "style" | "noscript" | "iframe" | "object" | "embed"
            ) {
                return;
            }

            // Recursively process children
            for child in handle.children.borrow().iter() {
                extract_text_recursive(child, text);
            }
        }
        _ => {
            // For Document, Doctype, Comment, etc., process children
            for child in handle.children.borrow().iter() {
                extract_text_recursive(child, text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_parser_creation() {
        let parser = HtmlParser::new();
        assert_eq!(parser.max_nodes, MAX_DOM_NODES);
    }

    #[test]
    fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let html = "<html><body><p>Hello, world!</p></body></html>";
        let dom = parser.parse(html).unwrap();
        assert!(dom.node_count() > 0);
    }

    #[test]
    fn test_extract_text() {
        let parser = HtmlParser::new();
        let html = "<html><body><h1>Title</h1><p>Paragraph text.</p></body></html>";
        let dom = parser.parse(html).unwrap();
        let text = dom.extract_text();
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph text"));
    }

    #[test]
    fn test_filter_script_tags() {
        let parser = HtmlParser::new();
        let html = r#"
            <html>
                <head><script>alert('test');</script></head>
                <body><p>Visible text</p></body>
            </html>
        "#;
        let dom = parser.parse(html).unwrap();
        let text = dom.extract_text();
        assert!(!text.contains("alert"));
        assert!(text.contains("Visible text"));
    }

    #[test]
    fn test_malformed_html() {
        let parser = HtmlParser::new();
        // html5ever handles malformed HTML gracefully
        let html = "<p>Unclosed paragraph<div>Some text";
        let result = parser.parse(html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_nodes_limit() {
        let parser = HtmlParser::with_max_nodes(10);
        let html = "<div><p>1</p><p>2</p><p>3</p><p>4</p><p>5</p><p>6</p></div>";
        let result = parser.parse(html);
        // With a low limit, this should exceed the node count
        assert!(result.is_err() || result.unwrap().node_count() <= 10);
    }
}
