//! Inline content rendering with theme-based formatting.

use crate::theme::Theme;
use streamdown_parser::{decode_html_entities, InlineElement, InlineParser};

/// Render inline elements to a string using the theme.
pub fn render_inline_content(content: &str, theme: &Theme) -> String {
    let mut parser = InlineParser::new();
    let elements = parser.parse(content);

    let mut result = String::new();

    for element in elements {
        match element {
            InlineElement::Text(text) => {
                result.push_str(&decode_html_entities(&text));
            }
            InlineElement::Bold(text) => {
                result.push_str(&theme.bold.apply(&decode_html_entities(&text)).to_string());
            }
            InlineElement::Italic(text) => {
                result.push_str(&theme.italic.apply(&decode_html_entities(&text)).to_string());
            }
            InlineElement::BoldItalic(text) => {
                // Combine bold and italic
                let decoded = decode_html_entities(&text);
                let styled = theme.bold.apply(&decoded);
                result.push_str(&theme.italic.apply(&styled.to_string()).to_string());
            }
            InlineElement::Strikeout(text) => {
                result.push_str(&theme.strikethrough.apply(&decode_html_entities(&text)).to_string());
            }
            InlineElement::Underline(text) => {
                // Use bold with underline effect
                let decoded = decode_html_entities(&text);
                result.push_str(&format!("\x1b[4m{}\x1b[24m", decoded));
            }
            InlineElement::Code(text) => {
                result.push_str(&theme.code.apply(&text).to_string());
            }
            InlineElement::Link { text, url } => {
                // OSC 8 hyperlink
                result.push_str("\x1b]8;;");
                result.push_str(&url);
                result.push_str("\x1b\\");
                result.push_str(&theme.link.apply(&decode_html_entities(&text)).to_string());
                result.push_str("\x1b]8;;\x1b\\");
                result.push(' ');
                result.push_str(&theme.link_url.apply(&format!("({})", url)).to_string());
            }
            InlineElement::Image { alt, .. } => {
                result.push_str(&format!("[🖼 {}]", alt));
            }
            InlineElement::Footnote(text) => {
                result.push_str(&text);
            }
        }
    }

    result
}
