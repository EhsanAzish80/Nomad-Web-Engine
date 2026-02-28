//! CSS parsing and style computation for the Nomad Web Engine.
//!
//! Handles CSS parsing, cascade resolution, and computed styles.
//! 
//! SCOPE: CSS-lite - only essential properties for Phase 5.
//! - No animations, transitions, calc(), variables
//! - No grid, no complex selectors, no pseudo-classes
//! - Silent ignoring of unsupported properties

use cssparser::{Parser, ParserInput, Token, ParseError};
use markup5ever_rcdom::{Handle, NodeData};
use thiserror::Error;

/// Safety limits for CSS.
pub const MAX_CSS_RULES: usize = 10_000;
pub const MAX_CSS_FILE_SIZE: usize = 1_048_576; // 1MB
pub const MAX_CSS_FILES: usize = 10;
pub const MAX_SELECTOR_DEPTH: usize = 10;

/// User-agent stylesheet with sensible defaults for Phase 6 usability.
const USER_AGENT_CSS: &str = r#"
/* Block-level elements */
html, body, div, article, section, nav, aside, header, footer, main {
    display: block;
}

/* Inline elements */
span, a, em, strong, code, b, i, u {
    display: inline;
}

/* Form elements with intrinsic sizing */
input {
    display: inline-block;
    width: 200px;
    height: 24px;
    padding: 4px;
    margin: 2px;
}

button {
    display: inline-block;
    padding: 6px 12px;
    margin: 2px;
    min-width: 60px;
    height: 32px;
}

/* Headings */
h1, h2, h3, h4, h5, h6 {
    display: block;
    margin: 8px 0;
}

/* Lists */
ul, ol {
    display: block;
    margin: 8px 0;
}

li {
    display: block;
}

/* Paragraphs */
p {
    display: block;
    margin: 8px 0;
}

/* Images should be inline-block */
img {
    display: inline-block;
}

/* Forms */
form {
    display: block;
}
"#;

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
    InlineBlock,
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

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
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
    pub text_align: TextAlign,
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
            text_align: TextAlign::Left,
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

/// Selector can be simple or compound (descendant combinator).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Simple(SimpleSelector),
    Descendant(Vec<SimpleSelector>), // e.g., "div p" = [Tag("div"), Tag("p")]
}

impl Selector {
    /// Get the rightmost (target) selector for specificity.
    pub fn target(&self) -> &SimpleSelector {
        match self {
            Selector::Simple(s) => s,
            Selector::Descendant(parts) => parts.last().unwrap(),
        }
    }
    
    /// Get depth of selector (number of parts).
    pub fn depth(&self) -> usize {
        match self {
            Selector::Simple(_) => 1,
            Selector::Descendant(parts) => parts.len(),
        }
    }
}

/// A CSS rule with a selector and declarations.
#[derive(Debug, Clone)]
pub struct Rule {
    pub selector: Selector,
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
    TextAlign(TextAlign),
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
    /// Enforces MAX_CSS_RULES limit for security.
    pub fn parse(css: &str) -> Self {
        let mut stylesheet = Self::new();
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);

        while !parser.is_exhausted() {
            // Enforce MAX_CSS_RULES limit
            if stylesheet.rules.len() >= MAX_CSS_RULES {
                break;
            }
            
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

    /// Parse CSS with user-agent stylesheet prepended.
    /// User-agent rules come first, then author CSS, following CSS cascade order.
    /// Silently ignores unsupported rules and properties.
    /// Enforces MAX_CSS_RULES limit for security.
    pub fn parse_with_user_agent(css: &str) -> Self {
        // Combine user-agent stylesheet with author CSS
        let combined = format!("{}\n{}", USER_AGENT_CSS, css);
        Self::parse(&combined)
    }

    /// Apply styles to a DOM node and compute final styles.
    pub fn compute_style(&self, node: &Handle, parent_style: Option<&ComputedStyle>) -> ComputedStyle {
        let mut style = parent_style.cloned().unwrap_or_default();

        // Extract element info
        let (tag_name, classes, id) = extract_element_info(node);

        // Apply matching rules (in order, later rules override earlier)
        for rule in &self.rules {
            if matches_selector(&rule.selector, node, &tag_name, &classes, &id) {
                apply_declarations(&mut style, &rule.declarations);
            }
        }

        style
    }
}

/// Extract tag, classes, and id from a node.
fn extract_element_info(node: &Handle) -> (Option<String>, Vec<String>, Option<String>) {
    match &node.data {
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
    }
}

/// Check if a selector matches a node.
fn matches_selector(
    selector: &Selector,
    node: &Handle,
    tag_name: &Option<String>,
    classes: &[String],
    id: &Option<String>,
) -> bool {
    match selector {
        Selector::Simple(simple) => matches_simple(simple, tag_name, classes, id),
        Selector::Descendant(parts) => {
            // For descendant selectors, check from right to left
            // The rightmost selector must match the current node
            if parts.is_empty() {
                return false;
            }

            let target = &parts[parts.len() - 1];
            if !matches_simple(target, tag_name, classes, id) {
                return false;
            }

            // If only one part, it's actually a simple selector
            if parts.len() == 1 {
                return true;
            }

            // Check ancestors for remaining selectors (right to left)
            matches_ancestors(node, &parts[..parts.len() - 1])
        }
    }
}

/// Check if a simple selector matches the element info.
fn matches_simple(
    selector: &SimpleSelector,
    tag_name: &Option<String>,
    classes: &[String],
    id: &Option<String>,
) -> bool {
    match selector {
        SimpleSelector::Universal => true,
        SimpleSelector::Tag(tag) => tag_name.as_ref() == Some(tag),
        SimpleSelector::Class(class) => classes.contains(class),
        SimpleSelector::Id(rule_id) => id.as_ref() == Some(rule_id),
    }
}

/// Check if ancestors match the selector parts (from right to left).
/// Example: for "div p span", when checking "span", this checks if ancestors contain "p" with "div" ancestor.
fn matches_ancestors(node: &Handle, parts: &[SimpleSelector]) -> bool {
    if parts.is_empty() {
        return true;
    }

    // Walk up the ancestor chain
    let mut current = node.parent.take();
    
    // Find the next selector match in ancestors
    let target = &parts[parts.len() - 1];
    let remaining = &parts[..parts.len() - 1];
    
    while let Some(ref parent_weak) = current {
        if let Some(parent) = parent_weak.upgrade() {
            let (tag, classes, id) = extract_element_info(&parent);
            
            if matches_simple(target, &tag, &classes, &id) {
                // Found a match for this selector
                // Continue checking remaining selectors up the ancestor chain
                if remaining.is_empty() {
                    // Restore parent reference
                    node.parent.set(current.take());
                    return true;
                }
                
                // Recursively check remaining ancestors
                let result = matches_ancestors(&parent, remaining);
                node.parent.set(current.take());
                return result;
            }
            
            // Continue searching up the tree
            current = parent.parent.take();
        } else {
            break;
        }
    }
    
    // Restore parent reference
    node.parent.set(current);
    false
}

/// Parse a single CSS rule.
fn parse_rule<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Rule, ParseError<'i, ()>> {
    // Parse selector (simple or descendant)
    let selector = parse_selector(parser)?;

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

/// Parse a selector (simple or descendant combinator).
fn parse_selector<'i, 't>(parser: &mut Parser<'i, 't>) -> Result<Selector, ParseError<'i, ()>> {
    let mut parts = Vec::new();
    
    // Parse first simple selector
    parts.push(parse_simple_selector(parser)?);
    
    // Check for descendant combinator (whitespace followed by another selector)
    loop {
        let _ = parser.skip_whitespace();
        
        // Try to parse another selector using try_parse to avoid consuming on failure
        let result = parser.try_parse(|p| parse_simple_selector(p));
        
        match result {
            Ok(part) => {
                parts.push(part);
                
                // Enforce max selector depth
                if parts.len() > MAX_SELECTOR_DEPTH {
                    return Err(parser.new_custom_error(()));
                }
            }
            Err(_) => {
                // No more selectors, done
                break;
            }
        }
    }
    
    // Return simple or descendant selector
    if parts.len() == 1 {
        Ok(Selector::Simple(parts.into_iter().next().unwrap()))
    } else {
        Ok(Selector::Descendant(parts))
    }
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
                "inline-block" => Display::InlineBlock,
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
        "text-align" => {
            let ident = parser.expect_ident()?.to_string().to_lowercase();
            let align = match ident.as_str() {
                "left" => TextAlign::Left,
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                _ => return Err(parser.new_custom_error(())),
            };
            Ok(PropertyValue::TextAlign(align))
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
            "text-align" => {
                if let PropertyValue::TextAlign(a) = decl.value {
                    style.text_align = a;
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
        assert_eq!(stylesheet.rules[0].selector, Selector::Simple(SimpleSelector::Tag("div".to_string())));
        assert_eq!(stylesheet.rules[0].declarations.len(), 1);

        // Check second rule
        assert_eq!(stylesheet.rules[1].selector, Selector::Simple(SimpleSelector::Class("container".to_string())));

        // Check third rule
        assert_eq!(stylesheet.rules[2].selector, Selector::Simple(SimpleSelector::Id("main".to_string())));
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
        assert_eq!(style.text_align, TextAlign::Left);
    }

    #[test]
    fn test_descendant_selector_parsing() {
        let css = r#"div p { font-size: 14px; }"#;

        let stylesheet = StyleSheet::parse(css);
        assert_eq!(stylesheet.rules.len(), 1);

        // Check first rule (div p)
        match &stylesheet.rules[0].selector {
            Selector::Descendant(parts) => {
                assert_eq!(parts.len(), 2);
                assert_eq!(parts[0], SimpleSelector::Tag("div".to_string()));
                assert_eq!(parts[1], SimpleSelector::Tag("p".to_string()));
            }
            _ => panic!("Expected descendant selector, got: {:?}", stylesheet.rules[0].selector),
        }
    }

    #[test]
    fn test_text_align_parsing() {
        let css = r#"
            .left { text-align: left; }
            .center { text-align: center; }
            .right { text-align: right; }
        "#;

        let stylesheet = StyleSheet::parse(css);
        assert_eq!(stylesheet.rules.len(), 3);
        
        // Verify all rules have text-align declarations
        for rule in &stylesheet.rules {
            assert!(rule.declarations.iter().any(|d| d.property == "text-align"));
        }
    }
}
