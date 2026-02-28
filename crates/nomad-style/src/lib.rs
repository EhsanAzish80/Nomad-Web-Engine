//! CSS parsing and style computation for the Nomad Web Engine.
//!
//! Handles CSS parsing, cascade resolution, and computed styles.
//! 
//! SCOPE: CSS-lite - only essential properties for Phase 4.
//! - No animations, transitions, calc(), variables
//! - No grid, no complex selectors, no pseudo-classes
//! - Silent ignoring of unsupported properties

use cssparser::{Parser, ParserInput, Token, ParseError};
use markup5ever_rcdom::{Handle, NodeData};
use thiserror::Error;

/// Errors that can occur during style processing.
#[derive(Error, Debug)]
pub enum StyleError {
    #[error("CSS parsing failed: {0}")]
    ParseError(String),
}

/// Display type for an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    #[default]
    Block,
    Inline,
    Flex,
    None,
}

/// Flex direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

/// Flex justify-content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
}

/// Flex align-items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
}

/// Edge insets (margin or padding).
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

/// Computed style for an element.
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub display: Display,
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub font_size: f32,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Block,
            margin: EdgeInsets::zero(),
            padding: EdgeInsets::zero(),
            width: None,
            height: None,
            font_size: 16.0,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
        }
    }
}

/// Simple selector (tag, class, or id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimpleSelector {
    Tag(String),
    Class(String),
    Id(String),
    Universal,
}

/// A CSS rule with a simple selector and declarations.
#[derive(Debug, Clone)]
pub struct Rule {
    pub selector: SimpleSelector,
    pub declarations: Vec<Declaration>,
}

/// A CSS declaration (property: value).
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: PropertyValue,
}

/// CSS property values we support.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Display(Display),
    FlexDirection(FlexDirection),
    JustifyContent(JustifyContent),
    AlignItems(AlignItems),
    Length(f32), // Only px for now
    Auto,
}

/// CSS stylesheet representation.
#[derive(Debug, Clone, Default)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse CSS from a string.
    /// Silently ignores unsupported rules and properties.
    pub fn parse(css: &str) -> Self {
        let mut stylesheet = Self::new();
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);

        while !parser.is_exhausted() {
            // Skip whitespace and comments
            let _ = parser.skip_whitespace();
            
            if parser.is_exhausted() {
                break;
            }

            // Try to parse a rule
            if let Ok(rule) = parse_rule(&mut parser) {
                stylesheet.rules.push(rule);
            } else {
                // Skip to next rule on error (silent ignore)
                skip_to_next_rule(&mut parser);
            }
        }

        stylesheet
    }

    /// Apply styles to a DOM node and compute final styles.
    pub fn compute_style(&self, node: &Handle, parent_style: Option<&ComputedStyle>) -> ComputedStyle {
        let mut style = parent_style.cloned().unwrap_or_default();

        // Extract element info
        let (tag_name, classes, id) = match &node.data {
            NodeData::Element { ref name, ref attrs, .. } => {
                let tag = name.local.as_ref().to_lowercase();
                let mut classes = Vec::new();
                let mut id = None;

                for attr in attrs.borrow().iter() {
                    match attr.name.local.as_ref() {
                        "class" => {
                            classes = attr.value.split_whitespace().map(|s| s.to_string()).collect();
                        }
                        "id" => {
                            id = Some(attr.value.to_string());
                        }
                        _ => {}
                    }
                }

                (Some(tag), classes, id)
            }
            _ => (None, Vec::new(), None),
        };

        // Apply matching rules (in order, later rules override earlier)
        for rule in &self.rules {
            let matches = match &rule.selector {
                SimpleSelector::Universal => true,
                SimpleSelector::Tag(tag) => tag_name.as_ref() == Some(tag),
                SimpleSelector::Class(class) => classes.contains(class),
                SimpleSelector::Id(rule_id) => id.as_ref() == Some(rule_id),
            };

            if matches {
                apply_declarations(&mut style, &rule.declarations);
            }
        }

        style
    }
}

/// Parse a single CSS rule.
fn parse_rule<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Rule, ParseError<'i, ()>> {
    // Parse selector
    let selector = parse_simple_selector(parser)?;

    // Expect '{'
    parser.expect_curly_bracket_block()?;

    // Parse declarations
    let declarations = parser.parse_nested_block(|parser| {
        let mut decls = Vec::new();
        
        while !parser.is_exhausted() {
            let _ = parser.skip_whitespace();
            
            if parser.is_exhausted() {
                break;
            }

            // Try to parse declaration
            if let Ok(decl) = parse_declaration(parser) {
                decls.push(decl);
            } else {
                // Skip to next declaration
                skip_to_next_declaration(parser);
            }
        }

        Ok(decls)
    })?;

    Ok(Rule {
        selector,
        declarations,
    })
}

/// Parse a simple selector (tag, .class, or #id).
fn parse_simple_selector<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<SimpleSelector, ParseError<'i, ()>> {
    let token = parser.next()?.clone();

    match &token {
        Token::Ident(name) => {
            // Tag selector
            Ok(SimpleSelector::Tag(name.to_string().to_lowercase()))
        }
        Token::Delim('.') => {
            // Class selector
            let class_name = parser.expect_ident()?.to_string();
            Ok(SimpleSelector::Class(class_name))
        }
        Token::IDHash(id) => {
            // ID selector
            Ok(SimpleSelector::Id(id.to_string()))
        }
        Token::Delim('*') => {
            // Universal selector
            Ok(SimpleSelector::Universal)
        }
        _ => {
            Err(parser.new_unexpected_token_error(token))
        }
    }
}

/// Parse a declaration (property: value;).
fn parse_declaration<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Declaration, ParseError<'i, ()>> {
    let property = parser.expect_ident()?.to_string().to_lowercase();
    parser.expect_colon()?;
    
    let value = parse_property_value(parser, &property)?;
    
    // Optional semicolon
    let _ = parser.try_parse(|p| p.expect_semicolon());

    Ok(Declaration { property, value })
}

/// Parse a property value based on property name.
/// Returns Err for unsupported properties (which are silently ignored).
fn parse_property_value<'i, 't>(parser: &mut Parser<'i, 't>, property: &str) -> Result<PropertyValue, ParseError<'i, ()>> {
    match property {
        "display" => {
            let ident = parser.expect_ident()?.to_string().to_lowercase();
            let display = match ident.as_str() {
                "block" => Display::Block,
                "inline" => Display::Inline,
                "flex" => Display::Flex,
                "none" => Display::None,
                _ => return Err(parser.new_custom_error(())),
            };
            Ok(PropertyValue::Display(display))
        }
        "flex-direction" => {
            let ident = parser.expect_ident()?.to_string().to_lowercase();
            let direction = match ident.as_str() {
                "row" => FlexDirection::Row,
                "column" => FlexDirection::Column,
                _ => return Err(parser.new_custom_error(())),
            };
            Ok(PropertyValue::FlexDirection(direction))
        }
        "justify-content" => {
            let ident = parser.expect_ident()?.to_string().to_lowercase();
            let justify = match ident.as_str() {
                "flex-start" => JustifyContent::FlexStart,
                "flex-end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                _ => return Err(parser.new_custom_error(())),
            };
            Ok(PropertyValue::JustifyContent(justify))
        }
        "align-items" => {
            let ident = parser.expect_ident()?.to_string().to_lowercase();
            let align = match ident.as_str() {
                "stretch" => AlignItems::Stretch,
                "flex-start" => AlignItems::FlexStart,
                "flex-end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                _ => return Err(parser.new_custom_error(())),
            };
            Ok(PropertyValue::AlignItems(align))
        }
        "margin" | "padding" | "width" | "height" | "font-size" |
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" |
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
            parse_length_or_auto(parser)
        }
        _ => {
            // Unsupported property - silently ignore
            Err(parser.new_custom_error(()))
        }
    }
}

/// Parse a length value (only px supported) or auto.
fn parse_length_or_auto<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<PropertyValue, ParseError<'i, ()>> {
    let token = parser.next()?.clone();
    
    match &token {
        Token::Dimension { value, ref unit, .. } => {
            if unit.eq_ignore_ascii_case("px") {
                Ok(PropertyValue::Length(*value))
            } else {
                // Unsupported unit - silently ignore
                Err(parser.new_custom_error(()))
            }
        }
        Token::Number { value, .. } => {
            // Treat unitless as px for now
            Ok(PropertyValue::Length(*value))
        }
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
            Ok(PropertyValue::Auto)
        }
        _ => Err(parser.new_unexpected_token_error(token)),
    }
}

/// Apply declarations to a computed style.
fn apply_declarations(style: &mut ComputedStyle, declarations: &[Declaration]) {
    for decl in declarations {
        match decl.property.as_str() {
            "display" => {
                if let PropertyValue::Display(d) = decl.value {
                    style.display = d;
                }
            }
            "flex-direction" => {
                if let PropertyValue::FlexDirection(d) = decl.value {
                    style.flex_direction = d;
                }
            }
            "justify-content" => {
                if let PropertyValue::JustifyContent(j) = decl.value {
                    style.justify_content = j;
                }
            }
            "align-items" => {
                if let PropertyValue::AlignItems(a) = decl.value {
                    style.align_items = a;
                }
            }
            "margin" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.margin = EdgeInsets::all(v);
                }
            }
            "margin-top" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.margin.top = v;
                }
            }
            "margin-right" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.margin.right = v;
                }
            }
            "margin-bottom" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.margin.bottom = v;
                }
            }
            "margin-left" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.margin.left = v;
                }
            }
            "padding" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.padding = EdgeInsets::all(v);
                }
            }
            "padding-top" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.padding.top = v;
                }
            }
            "padding-right" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.padding.right = v;
                }
            }
            "padding-bottom" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.padding.bottom = v;
                }
            }
            "padding-left" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.padding.left = v;
                }
            }
            "width" => {
                style.width = match decl.value {
                    PropertyValue::Length(v) => Some(v),
                    PropertyValue::Auto => None,
                    _ => style.width,
                };
            }
            "height" => {
                style.height = match decl.value {
                    PropertyValue::Length(v) => Some(v),
                    PropertyValue::Auto => None,
                    _ => style.height,
                };
            }
            "font-size" => {
                if let PropertyValue::Length(v) = decl.value {
                    style.font_size = v;
                }
            }
            _ => {
                // Unknown property - silently ignore
            }
        }
    }
}

/// Skip to the next rule (used for error recovery).
fn skip_to_next_rule<'i, 't>(parser: &mut Parser<'i, 't>) {
    while !parser.is_exhausted() {
        let _ = parser.next();
        // Look for '}' to end current rule
        if matches!(parser.current_source_location(), _) {
            // Simple approach: just consume tokens until we're done
            break;
        }
    }
}

/// Skip to the next declaration (used for error recovery).
fn skip_to_next_declaration<'i, 't>(parser: &mut Parser<'i, 't>) {
    while !parser.is_exhausted() {
        if let Ok(token) = parser.next() {
            if matches!(token, Token::Semicolon) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_css() {
        let css = r#"
            div { display: flex; }
            .container { flex-direction: row; }
            #main { width: 800px; }
        "#;

        let stylesheet = StyleSheet::parse(css);
        assert_eq!(stylesheet.rules.len(), 3);

        // Check first rule
        assert_eq!(stylesheet.rules[0].selector, SimpleSelector::Tag("div".to_string()));
        assert_eq!(stylesheet.rules[0].declarations.len(), 1);

        // Check second rule
        assert_eq!(stylesheet.rules[1].selector, SimpleSelector::Class("container".to_string()));

        // Check third rule
        assert_eq!(stylesheet.rules[2].selector, SimpleSelector::Id("main".to_string()));
    }

    #[test]
    fn test_silent_ignore_unsupported() {
        let css = r#"
            div {
                display: flex;
                animation: slide 1s;
                transition: all 0.3s;
                grid-template-columns: 1fr 1fr;
            }
        "#;

        // Should parse without error, ignoring unsupported properties
        let stylesheet = StyleSheet::parse(css);
        assert_eq!(stylesheet.rules.len(), 1);
        // Only display: flex should be parsed
        assert_eq!(stylesheet.rules[0].declarations.len(), 1);
    }

    #[test]
    fn test_parse_margin_padding() {
        let css = r#"
            .box {
                margin: 10px;
                padding-top: 20px;
                padding-left: 15px;
            }
        "#;

        let stylesheet = StyleSheet::parse(css);
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations.len(), 3);
    }

    #[test]
    fn test_computed_style_defaults() {
        let style = ComputedStyle::default();
        assert_eq!(style.display, Display::Block);
        assert_eq!(style.font_size, 16.0);
        assert_eq!(style.flex_direction, FlexDirection::Row);
    }
}
