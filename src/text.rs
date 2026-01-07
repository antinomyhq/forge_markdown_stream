//! Text wrapping utilities with ANSI code awareness.

use unicode_width::UnicodeWidthChar;

/// Calculate visible length of a string (excluding ANSI escape codes).
/// Uses Unicode width to properly handle wide characters like emojis.
/// Handles both CSI sequences (\x1b[...m) and OSC sequences (\x1b]...\x1b\\).
pub fn visible_length(s: &str) -> usize {
    let mut len = 0;
    let mut in_csi = false; // CSI sequence: \x1b[...m
    let mut in_osc = false; // OSC sequence: \x1b]...\x1b\\
    let mut prev_was_esc = false;

    for c in s.chars() {
        if prev_was_esc {
            prev_was_esc = false;
            if c == '[' {
                in_csi = true;
                continue;
            } else if c == ']' {
                in_osc = true;
                continue;
            } else if c == '\\' && in_osc {
                // End of OSC sequence
                in_osc = false;
                continue;
            }
            // Not a recognized sequence, count the escape and this char
            len += c.width().unwrap_or(0);
            continue;
        }

        if c == '\x1b' {
            prev_was_esc = true;
            continue;
        }

        if in_csi {
            if c == 'm' || c == 'K' || c == 'H' || c == 'J' {
                in_csi = false;
            }
            continue;
        }

        if in_osc {
            // Skip all characters inside OSC sequence until we see \x1b\\
            continue;
        }

        len += c.width().unwrap_or(0);
    }

    len
}

/// Result of wrapping text.
pub struct WrappedText {
    pub lines: Vec<String>,
}

impl WrappedText {
    pub fn empty() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Split text into words while preserving ANSI codes.
pub fn split_text(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_escape = false;
    let mut escape_buf = String::new();

    for ch in text.chars() {
        if in_escape {
            escape_buf.push(ch);
            if ch == 'm' {
                current.push_str(&escape_buf);
                escape_buf.clear();
                in_escape = false;
            }
            continue;
        }

        if ch == '\x1b' {
            in_escape = true;
            escape_buf.push(ch);
            continue;
        }

        if ch.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if !escape_buf.is_empty() {
        current.push_str(&escape_buf);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Simple text wrap for plain text.
pub fn simple_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.is_empty() {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        let current_len = current.chars().count();

        if current.is_empty() {
            current = word.to_string();
        } else if current_len + 1 + word_len <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Wrap text to fit within a given width (ANSI-aware).
pub fn text_wrap(
    text: &str,
    width: usize,
    first_prefix: &str,
    next_prefix: &str,
) -> WrappedText {
    if width == 0 {
        return WrappedText::empty();
    }

    let words = split_text(text);
    if words.is_empty() {
        return WrappedText::empty();
    }

    let first_prefix_len = visible_length(first_prefix);
    let next_prefix_len = visible_length(next_prefix);

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0;
    let mut is_first_line = true;

    for word in &words {
        let word_len = visible_length(word);
        let prefix_len = if is_first_line {
            first_prefix_len
        } else {
            next_prefix_len
        };
        let available = width.saturating_sub(prefix_len);

        let space_needed = if current_line.is_empty() { 0 } else { 1 };

        if current_len + word_len + space_needed <= available {
            if !current_line.is_empty() {
                current_line.push(' ');
                current_len += 1;
            }
            current_line.push_str(word);
            current_len += word_len;
        } else {
            // Finalize current line
            if !current_line.is_empty() {
                let prefix = if is_first_line {
                    first_prefix
                } else {
                    next_prefix
                };
                lines.push(format!("{}{}", prefix, current_line));
                is_first_line = false;
            }
            // Start new line
            current_line = word.clone();
            current_len = word_len;
        }
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        let prefix = if is_first_line {
            first_prefix
        } else {
            next_prefix
        };
        lines.push(format!("{}{}", prefix, current_line));
    }

    WrappedText { lines }
}


#[cfg(test)]
mod tests {
    use super::*;

    // ==================== visible_length tests ====================

    #[test]
    fn test_visible_length_plain_text() {
        assert_eq!(visible_length("hello"), 5);
        assert_eq!(visible_length("hello world"), 11);
        assert_eq!(visible_length(""), 0);
    }

    #[test]
    fn test_visible_length_with_ansi() {
        // Bold text
        assert_eq!(visible_length("\x1b[1mhello\x1b[0m"), 5);
        // Colored text
        assert_eq!(visible_length("\x1b[31mred\x1b[0m"), 3);
        // Multiple codes
        assert_eq!(visible_length("\x1b[1m\x1b[31mbold red\x1b[0m"), 8);
    }

    #[test]
    fn test_visible_length_unicode() {
        // Wide characters (CJK)
        assert_eq!(visible_length("你好"), 4); // Each Chinese char is 2 wide
        // Emoji
        assert_eq!(visible_length("👋"), 2); // Emoji is typically 2 wide
    }

    #[test]
    fn test_visible_length_mixed_ansi_unicode() {
        assert_eq!(visible_length("\x1b[1m你好\x1b[0m"), 4);
    }

    // ==================== split_text tests ====================

    #[test]
    fn test_split_text_simple() {
        assert_eq!(split_text("hello world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_split_text_multiple_spaces() {
        assert_eq!(split_text("hello   world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_split_text_empty() {
        let result: Vec<String> = split_text("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_text_single_word() {
        assert_eq!(split_text("hello"), vec!["hello"]);
    }

    #[test]
    fn test_split_text_preserves_ansi() {
        let result = split_text("\x1b[1mhello\x1b[0m world");
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("\x1b[1m"));
        assert!(result[0].contains("hello"));
        assert_eq!(result[1], "world");
    }

    #[test]
    fn test_split_text_ansi_spans_word() {
        let result = split_text("\x1b[31mred text\x1b[0m");
        // ANSI code attaches to first word, second word is plain
        assert_eq!(result.len(), 2);
        assert!(result[0].starts_with("\x1b[31m"));
    }

    // ==================== simple_wrap tests ====================

    #[test]
    fn test_simple_wrap_no_wrap_needed() {
        let result = simple_wrap("hello world", 20);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn test_simple_wrap_single_line() {
        let result = simple_wrap("short", 10);
        assert_eq!(result, vec!["short"]);
    }

    #[test]
    fn test_simple_wrap_wraps_at_width() {
        let result = simple_wrap("hello world test", 11);
        assert_eq!(result, vec!["hello world", "test"]);
    }

    #[test]
    fn test_simple_wrap_multiple_lines() {
        let result = simple_wrap("one two three four five", 10);
        assert_eq!(result, vec!["one two", "three four", "five"]);
    }

    #[test]
    fn test_simple_wrap_empty() {
        let result = simple_wrap("", 10);
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_simple_wrap_zero_width() {
        let result = simple_wrap("hello", 0);
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn test_simple_wrap_long_word() {
        // Word longer than width stays on its own line
        let result = simple_wrap("superlongword short", 10);
        assert_eq!(result, vec!["superlongword", "short"]);
    }

    // ==================== text_wrap tests ====================

    #[test]
    fn test_text_wrap_with_prefix() {
        let result = text_wrap("hello world", 20, "  ", "  ");
        assert_eq!(result.lines, vec!["  hello world"]);
    }

    #[test]
    fn test_text_wrap_different_prefixes() {
        let result = text_wrap("one two three four", 12, "• ", "  ");
        assert_eq!(result.lines.len(), 2);
        assert!(result.lines[0].starts_with("• "));
        assert!(result.lines[1].starts_with("  "));
    }

    #[test]
    fn test_text_wrap_empty() {
        let result = text_wrap("", 20, "  ", "  ");
        assert!(result.is_empty());
    }

    #[test]
    fn test_text_wrap_zero_width() {
        let result = text_wrap("hello", 0, "  ", "  ");
        assert!(result.is_empty());
    }

    #[test]
    fn test_text_wrap_respects_prefix_width() {
        // Width 15, prefix ">>> " is 4 chars, so 11 chars available
        let result = text_wrap("hello world test", 15, ">>> ", "    ");
        assert_eq!(result.lines[0], ">>> hello world");
        assert_eq!(result.lines[1], "    test");
    }

    #[test]
    fn test_text_wrap_with_ansi() {
        let result = text_wrap("\x1b[1mbold\x1b[0m text", 20, "  ", "  ");
        assert_eq!(result.lines.len(), 1);
        assert!(result.lines[0].contains("\x1b[1m"));
    }

    #[test]
    fn test_text_wrap_preserves_ansi_across_lines() {
        let result = text_wrap("\x1b[1mone two three\x1b[0m", 10, "", "");
        assert!(result.lines.len() >= 2);
    }

    // ==================== WrappedText tests ====================

    #[test]
    fn test_wrapped_text_empty() {
        let wrapped = WrappedText::empty();
        assert!(wrapped.is_empty());
        assert!(wrapped.lines.is_empty());
    }

    #[test]
    fn test_wrapped_text_not_empty() {
        let wrapped = WrappedText {
            lines: vec!["hello".to_string()],
        };
        assert!(!wrapped.is_empty());
    }
}
