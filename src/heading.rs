//! Heading rendering with theme-based styling.

use crate::text::simple_wrap;
use crate::theme::Theme;

/// Render a heading with appropriate styling.
pub fn render_heading(level: u8, content: &str, width: usize, margin: &str, theme: &Theme) -> Vec<String> {
    let lines = simple_wrap(content, width);
    let mut result = Vec::new();

    for line in lines {
        let line_len = line.chars().count();
        let center_pad = (width.saturating_sub(line_len)) / 2;

        let formatted = match level {
            1 => {
                // H1: Bold, centered
                format!(
                    "{}\n{}{}{}",
                    margin,
                    margin,
                    " ".repeat(center_pad),
                    theme.heading1.apply(&line)
                )
            }
            2 => {
                // H2: Bold, bright color, centered
                format!(
                    "{}\n{}{}{}",
                    margin,
                    margin,
                    " ".repeat(center_pad),
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
