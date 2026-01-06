//! Heading rendering with theme-based styling.

use crate::inline::render_inline_content;
use crate::text::simple_wrap;
use crate::theme::Theme;

/// Render a heading with appropriate styling.
pub fn render_heading(level: u8, content: &str, width: usize, margin: &str, theme: &Theme) -> Vec<String> {
    // First render inline elements (bold, italic, etc.) in the content
    let rendered_content = render_inline_content(content, theme);
    let lines = simple_wrap(&rendered_content, width);
    let mut result = Vec::new();

    for line in lines {
        let formatted = match level {
            1 => {
                // H1: Bold, left-aligned
                format!(
                    "{}\n{}{}",
                    margin,
                    margin,
                    theme.heading1.apply(&line)
                )
            }
            2 => {
                // H2: Bold, bright color, left-aligned
                format!(
                    "{}\n{}{}",
                    margin,
                    margin,
                    theme.heading2.apply(&line)
                )
            }
            3 => {
                format!("{}{}", margin, theme.heading3.apply(&line))
            }
            4 => {
                format!("{}{}", margin, theme.heading4.apply(&line))
            }
            5 => {
                format!("{}{}", margin, theme.heading5.apply(&line))
            }
            _ => {
                format!("{}{}", margin, theme.heading6.apply(&line))
            }
        };
        result.push(formatted);
    }

    result
}
