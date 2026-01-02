//! Demo of streaming LLM output with modular markdown rendering.

mod code;
mod heading;
mod inline;
mod list;
mod renderer;
mod table;
mod text;
mod theme;

use std::io::{self, Write};
use std::time::Duration;

use streamdown_parser::Parser;

use renderer::Renderer;
pub use theme::Theme;

/// Streaming renderer that buffers tokens and renders complete lines.
struct StreamdownRenderer<W: Write> {
    parser: Parser,
    renderer: Renderer<W>,
    line_buffer: String,
}

impl<W: Write> StreamdownRenderer<W> {
    fn new(writer: W, width: usize) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::new(writer, width),
            line_buffer: String::new(),
        }
    }

    #[allow(dead_code)]
    fn with_theme(writer: W, width: usize, theme: Theme) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::with_theme(writer, width, theme),
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

    // Use default dark theme, or Theme::light() for light theme
    let mut renderer = StreamdownRenderer::new(io::stdout(), width);

    for token in &tokens {
        renderer.push(token)?;
        io::stdout().flush()?;
        std::thread::sleep(Duration::from_millis(5));
    }
    renderer.finish()?;
    Ok(())
}
