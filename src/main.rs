//! Demo of streaming LLM output using streamdown crates.
//!
//! Run with: cargo run --example llm_stream

use std::io::{self, Write};
use std::time::Duration;

use streamdown_parser::{InlineParser, ParseEvent, Parser, ListBullet, InlineElement, format_line};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use termimad::crossterm::style::{Attribute, Color};
use termimad::{CompoundStyle, LineStyle, MadSkin};

/// Custom termimad-based renderer for streamdown events.
struct TermimadRenderer<W: Write> {
    writer: W,
    skin: MadSkin,
    width: usize,
    // Syntax highlighting
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_language: Option<String>,
    // Table buffering (first row is header)
    table_rows: Vec<Vec<String>>,
}

impl<W: Write> TermimadRenderer<W> {
    fn new(writer: W, width: usize) -> Self {
        let mut skin = MadSkin::default();
        let compound_style = CompoundStyle::new(Some(Color::Cyan), None, Default::default());
        skin.inline_code = compound_style.clone();

        let codeblock_style = CompoundStyle::new(None, None, Default::default());
        skin.code_block = LineStyle::new(codeblock_style, Default::default());

        let mut strikethrough_style = CompoundStyle::with_attr(Attribute::CrossedOut);
        strikethrough_style.add_attr(Attribute::Dim);
        skin.strikeout = strikethrough_style;

        Self {
            writer,
            skin,
            width,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_language: None,
            table_rows: Vec::new(),
        }
    }

    fn flush_table(&mut self) -> io::Result<()> {
        if self.table_rows.is_empty() {
            return Ok(());
        }
        let rows = std::mem::take(&mut self.table_rows);

        // Calculate column widths
        let mut widths: Vec<usize> = Vec::new();
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i >= widths.len() {
                    widths.push(cell.len());
                } else {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Top border
        let top: String = widths.iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┬");
        writeln!(self.writer, "┌{}┐", top)?;

        for (row_idx, row) in rows.iter().enumerate() {
            let is_header = row_idx == 0;
            let cells: String = row.iter()
                .enumerate()
                .map(|(i, c)| {
                    let w = widths.get(i).copied().unwrap_or(c.len());
                    // Format inline markdown (bold, italic, code, links, etc.)
                    let formatted = format_line(c, true, true);
                    if is_header {
                        format!(" \x1b[1m{:^w$}\x1b[22m ", formatted, w = w)
                    } else {
                        format!(" {:^w$} ", formatted, w = w)
                    }
                })
                .collect::<Vec<_>>()
                .join("│");
            writeln!(self.writer, "│{}│", cells)?;

            // Separator after header
            if is_header && rows.len() > 1 {
                let sep: String = widths.iter()
                    .map(|&w| "─".repeat(w + 2))
                    .collect::<Vec<_>>()
                    .join("┼");
                writeln!(self.writer, "├{}┤", sep)?;
            }
        }

        // Bottom border
        let bottom: String = widths.iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┴");
        writeln!(self.writer, "└{}┘", bottom)?;

        Ok(())
    }

    /// Highlight a line of code using syntect.
    fn highlight_code(&self, line: &str, language: Option<&str>) -> String {
        let syntax = language
            .and_then(|lang| self.syntax_set.find_syntax_by_token(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        match highlighter.highlight_line(line, &self.syntax_set) {
            Ok(ranges) => as_24_bit_terminal_escaped(&ranges[..], false),
            Err(_) => line.to_string(),
        }
    }

    fn render_event(&mut self, event: &ParseEvent) -> io::Result<()> {
        match event {
            ParseEvent::Text(text) => {
                write!(self.writer, "{}", text)?;
            }
            ParseEvent::InlineCode(code) => {
                // Cyan color for inline code
                write!(self.writer, "\x1b[36m{}\x1b[0m", code)?;
            }
            ParseEvent::Bold(text) => {
                write!(self.writer, "\x1b[1m{}\x1b[22m", text)?;
            }
            ParseEvent::Italic(text) => {
                write!(self.writer, "\x1b[3m{}\x1b[23m", text)?;
            }
            ParseEvent::BoldItalic(text) => {
                write!(self.writer, "\x1b[1m\x1b[3m{}\x1b[23m\x1b[22m", text)?;
            }
            ParseEvent::Underline(text) | ParseEvent::Prompt(text) => {
                write!(self.writer, "{}", text)?;
            }
            ParseEvent::Strikeout(text) => {
                write!(self.writer, "\x1b[9m\x1b[2m{}\x1b[22m\x1b[29m", text)?;
            }
            ParseEvent::Link { text, url } => {
                // OSC 8 hyperlink
                write!(self.writer, "\x1b]8;;{}\x1b\\\x1b[4m{}\x1b[24m\x1b]8;;\x1b\\", url, text)?;
            }
            ParseEvent::Image { alt, url } => {
                write!(self.writer, "![{}]({})", alt, url)?;
            }
            ParseEvent::Footnote(text) => {
                write!(self.writer, "[^{}]", text)?;
            }
            ParseEvent::Heading { level, content } => {
                let prefix = "#".repeat(*level as usize);
                let md = format!("{} {}", prefix, content);
                let formatted = self.skin.term_text(&md);
                write!(self.writer, "{}", formatted)?;
            }
            ParseEvent::CodeBlockStart { language, .. } => {
                self.current_language = language.clone();
            }
            ParseEvent::CodeBlockLine(line) => {
                let highlighted = self.highlight_code(line, self.current_language.as_deref());
                writeln!(self.writer, "{}\x1b[0m", highlighted)?;
            }
            ParseEvent::CodeBlockEnd => {
                self.current_language = None;
            }
            ParseEvent::ListItem { indent, bullet, content } => {
                let spaces = " ".repeat(*indent * 2);
                let marker = match bullet {
                    ListBullet::Dash => "•".to_string(),
                    ListBullet::Asterisk => "•".to_string(),
                    ListBullet::Plus => "•".to_string(),
                    ListBullet::PlusExpand => "+---".to_string(),
                    ListBullet::Ordered(n) => format!("{}.", n),
                };
                write!(self.writer, "{}{} ", spaces, marker)?;
                self.render_inline(content)?;
                writeln!(self.writer)?;
            }
            ParseEvent::ListEnd => {}
            ParseEvent::TableHeader(cols) | ParseEvent::TableRow(cols) => {
                self.table_rows.push(cols.clone());
            }
            ParseEvent::TableSeparator => {}
            ParseEvent::TableEnd => {
                self.flush_table()?;
            }
            ParseEvent::BlockquoteStart { .. } | ParseEvent::BlockquoteEnd => {}
            ParseEvent::BlockquoteLine(text) => {
                writeln!(self.writer, "\x1b[90m│\x1b[0m {}", text)?;
            }
            ParseEvent::ThinkBlockStart => {
                writeln!(self.writer, "\x1b[90m┌─ thinking ─\x1b[0m")?;
            }
            ParseEvent::ThinkBlockLine(line) => {
                writeln!(self.writer, "\x1b[90m│\x1b[0m {}", line)?;
            }
            ParseEvent::ThinkBlockEnd => {
                writeln!(self.writer, "\x1b[90m└\x1b[0m")?;
            }
            ParseEvent::HorizontalRule => {
                writeln!(self.writer, "{}", "─".repeat(self.width.min(40)))?;
            }
            ParseEvent::EmptyLine | ParseEvent::Newline => {
                writeln!(self.writer)?;
            }
            ParseEvent::InlineElements(elements) => {
                use streamdown_parser::InlineElement;
                for elem in elements {
                    match elem {
                        InlineElement::Text(t) => write!(self.writer, "{}", t)?,
                        InlineElement::Bold(t) => write!(self.writer, "\x1b[1m{}\x1b[22m", t)?,
                        InlineElement::Italic(t) => write!(self.writer, "\x1b[3m{}\x1b[23m", t)?,
                        InlineElement::BoldItalic(t) => write!(self.writer, "\x1b[1m\x1b[3m{}\x1b[23m\x1b[22m", t)?,
                        InlineElement::Underline(t) => write!(self.writer, "{}", t)?,
                        InlineElement::Strikeout(t) => write!(self.writer, "\x1b[9m\x1b[2m{}\x1b[22m\x1b[29m", t)?,
                        InlineElement::Code(t) => write!(self.writer, "\x1b[36m{}\x1b[0m", t)?,
                        InlineElement::Link { text, url } => {
                            write!(self.writer, "\x1b]8;;{}\x1b\\\x1b[4m{}\x1b[24m\x1b]8;;\x1b\\", url, text)?;
                        }
                        InlineElement::Image { alt, url } => write!(self.writer, "![{}]({})", alt, url)?,
                        InlineElement::Footnote(t) => write!(self.writer, "[^{}]", t)?,
                    }
                }
            }
        }
        Ok(())
    }

    /// Render text with inline markdown formatting using InlineParser.
    fn render_inline(&mut self, text: &str) -> io::Result<()> {
        let mut parser = InlineParser::new();
        let elements = parser.parse(text);

        for elem in elements {
            match elem {
                InlineElement::Text(t) => write!(self.writer, "{}", t)?,
                InlineElement::Bold(t) => write!(self.writer, "\x1b[1m{}\x1b[22m", t)?,
                InlineElement::Italic(t) => write!(self.writer, "\x1b[3m{}\x1b[23m", t)?,
                InlineElement::BoldItalic(t) => write!(self.writer, "\x1b[1m\x1b[3m{}\x1b[23m\x1b[22m", t)?,
                InlineElement::Underline(t) => write!(self.writer, "\x1b[4m{}\x1b[24m", t)?,
                InlineElement::Strikeout(t) => write!(self.writer, "\x1b[9m{}\x1b[29m", t)?,
                InlineElement::Code(t) => write!(self.writer, "\x1b[36m{}\x1b[0m", t)?,
                InlineElement::Link { text, url } => {
                    write!(self.writer, "\x1b]8;;{}\x1b\\\x1b[4m{}\x1b[24m\x1b]8;;\x1b\\", url, text)?;
                }
                InlineElement::Image { alt, url } => write!(self.writer, "![{}]({})", alt, url)?,
                InlineElement::Footnote(t) => write!(self.writer, "[^{}]", t)?,
            }
        }
        Ok(())
    }
}
struct StreamdownRenderer<W: Write> {
    parser: Parser,
    renderer: TermimadRenderer<W>,
    line_buffer: String,
}

impl<W: Write> StreamdownRenderer<W> {
    fn new(writer: W, width: usize) -> Self {
        Self {
            parser: Parser::new(),
            renderer: TermimadRenderer::new(writer, width),
            line_buffer: String::new(),
        }
    }

    fn push(&mut self, token: &str) -> io::Result<()> {
        self.line_buffer.push_str(token);

        while let Some(pos) = self.line_buffer.find('\n') {
            let line = self.line_buffer[..pos].to_string();
            for event in self.parser.parse_line(&line) {
                self.renderer.render_event(&event)?;
            }
            self.line_buffer = self.line_buffer[pos + 1..].to_string();
        }
        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        if !self.line_buffer.is_empty() {
            for event in self.parser.parse_line(&self.line_buffer) {
                self.renderer.render_event(&event)?;
            }
        }
        for event in self.parser.finalize() {
            self.renderer.render_event(&event)?;
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let content = include_str!(
        "/Users/ranjit/Desktop/workspace/forge/plans/2025-04-02-system-context-rendering-final.md"
    );
    let tokens: Vec<&str> = content.split_inclusive(" ").collect();
    let width = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(188);
    let mut renderer = StreamdownRenderer::new(io::stdout(), width);
    for token in &tokens {
        renderer.push(token)?;
        io::stdout().flush()?;
        std::thread::sleep(Duration::from_millis(5));
    }
    renderer.finish()?;
    Ok(())
}
