//! Theme configuration for markdown rendering.
//!
//! Provides customizable styling for all markdown elements using the `colored` crate.

use colored::{Color, ColoredString, Colorize};

/// Style configuration for a single element.
#[derive(Clone, Debug)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dimmed: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            dimmed: false,
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn dimmed(mut self) -> Self {
        self.dimmed = true;
        self
    }

    /// Apply this style to a string.
    pub fn apply(&self, text: &str) -> ColoredString {
        let mut result = text.normal();

        if let Some(fg) = self.fg {
            result = result.color(fg);
        }
        if let Some(bg) = self.bg {
            result = result.on_color(bg);
        }
        if self.bold {
            result = result.bold();
        }
        if self.italic {
            result = result.italic();
        }
        if self.underline {
            result = result.underline();
        }
        if self.strikethrough {
            result = result.strikethrough();
        }
        if self.dimmed {
            result = result.dimmed();
        }

        result
    }
}

/// Theme containing styles for all markdown elements.
#[derive(Clone, Debug)]
pub struct Theme {
    // Inline styles
    pub bold: Style,
    pub italic: Style,
    pub code: Style,
    pub strikethrough: Style,
    pub link: Style,
    pub link_url: Style,

    // Block styles
    pub heading1: Style,
    pub heading2: Style,
    pub heading3: Style,
    pub heading4: Style,
    pub heading5: Style,
    pub heading6: Style,

    // List styles
    pub bullet: Style,
    pub list_number: Style,
    pub checkbox_checked: Style,
    pub checkbox_unchecked: Style,

    // Table styles
    pub table_header: Style,
    pub table_border: Style,
    pub table_cell: Style,

    // Quote/Think styles
    pub blockquote: Style,
    pub blockquote_border: Style,
    pub think: Style,
    pub think_border: Style,

    // Code block
    pub code_block_lang: Style,

    // Horizontal rule
    pub hr: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default).
    pub fn dark() -> Self {
        Self {
            // Inline
            bold: Style::new().bold(),
            italic: Style::new().italic(),
            code: Style::new().fg(Color::Yellow),
            strikethrough: Style::new().strikethrough().dimmed(),
            link: Style::new().fg(Color::Cyan).underline(),
            link_url: Style::new().fg(Color::Blue).dimmed(),

            // Headings
            heading1: Style::new().fg(Color::Magenta).bold(),
            heading2: Style::new().fg(Color::Blue).bold(),
            heading3: Style::new().fg(Color::Cyan).bold(),
            heading4: Style::new().fg(Color::Green).bold(),
            heading5: Style::new().fg(Color::Yellow).bold(),
            heading6: Style::new().fg(Color::White).bold(),

            // Lists
            bullet: Style::new().fg(Color::Cyan),
            list_number: Style::new().fg(Color::Cyan),
            checkbox_checked: Style::new().fg(Color::Green),
            checkbox_unchecked: Style::new().fg(Color::Red),

            // Tables
            table_header: Style::new().bold(),
            table_border: Style::new().fg(Color::BrightBlack),
            table_cell: Style::new(),

            // Quotes
            blockquote: Style::new().italic().dimmed(),
            blockquote_border: Style::new().fg(Color::BrightBlack),
            think: Style::new().italic().fg(Color::BrightBlack),
            think_border: Style::new().fg(Color::BrightBlack),

            // Code block
            code_block_lang: Style::new().fg(Color::BrightBlack).italic(),

            // HR
            hr: Style::new().fg(Color::BrightBlack),
        }
    }

    /// Light theme for light terminal backgrounds.
    pub fn light() -> Self {
        Self {
            // Inline
            bold: Style::new().bold(),
            italic: Style::new().italic(),
            code: Style::new().fg(Color::Red),
            strikethrough: Style::new().strikethrough().dimmed(),
            link: Style::new().fg(Color::Blue).underline(),
            link_url: Style::new().fg(Color::Cyan).dimmed(),

            // Headings
            heading1: Style::new().fg(Color::Magenta).bold(),
            heading2: Style::new().fg(Color::Blue).bold(),
            heading3: Style::new().fg(Color::Cyan).bold(),
            heading4: Style::new().fg(Color::Green).bold(),
            heading5: Style::new().fg(Color::Yellow).bold(),
            heading6: Style::new().fg(Color::Black).bold(),

            // Lists
            bullet: Style::new().fg(Color::Blue),
            list_number: Style::new().fg(Color::Blue),
            checkbox_checked: Style::new().fg(Color::Green),
            checkbox_unchecked: Style::new().fg(Color::Red),

            // Tables
            table_header: Style::new().bold(),
            table_border: Style::new().fg(Color::Black),
            table_cell: Style::new(),

            // Quotes
            blockquote: Style::new().italic().dimmed(),
            blockquote_border: Style::new().fg(Color::Black),
            think: Style::new().italic().fg(Color::Black),
            think_border: Style::new().fg(Color::Black),

            // Code block
            code_block_lang: Style::new().fg(Color::Black).italic(),

            // HR
            hr: Style::new().fg(Color::Black),
        }
    }
}
