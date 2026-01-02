//! Demo of streaming LLM output with modular markdown rendering.

mod code;
mod heading;
mod inline;
mod list;
mod renderer;
mod table;
mod text;
mod theme;
mod writer;

use std::io;

use streamdown_parser::Parser;

use renderer::Renderer;
pub use theme::Theme;
pub use writer::StreamingWriter;

/// Streaming renderer that buffers tokens and renders complete lines.
/// Uses a background task for smooth character-by-character output.
pub struct StreamdownRenderer {
    parser: Parser,
    renderer: Renderer<StreamingWriter>,
    line_buffer: String,
}

impl StreamdownRenderer {
    pub fn new(width: usize) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::new(StreamingWriter::new(3), width),
            line_buffer: String::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_theme(width: usize, theme: Theme) -> Self {
        Self {
            parser: Parser::new(),
            renderer: Renderer::with_theme(StreamingWriter::new(3), width, theme),
            line_buffer: String::new(),
        }
    }

    pub fn push(&mut self, token: &str) -> io::Result<()> {
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

    /// Finish rendering and wait for all output to be written.
    pub async fn finish(mut self) -> io::Result<()> {
        if !self.line_buffer.is_empty() {
            for event in self.parser.parse_line(&self.line_buffer) {
                self.renderer.render_event(&event)?;
            }
        }
        for event in self.parser.finalize() {
            self.renderer.render_event(&event)?;
        }
        self.renderer.into_writer().finish().await
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let content = include_str!(
        "/Users/ranjit/Desktop/workspace/forge/plans/2025-04-02-system-context-rendering-final.md"
    );
    let tokens: Vec<&str> = content.split_inclusive(" ").collect();
    let width = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(188);

    let mut renderer = StreamdownRenderer::new(width);

    for token in &tokens {
        renderer.push(token)?;
    }

    renderer.finish().await?;

    Ok(())
}
